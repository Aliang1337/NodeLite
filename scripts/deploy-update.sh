#!/usr/bin/env bash
# deploy-update.sh — 将本地交叉编译的 nodelite-server 二进制部署到远端服务器
#
# 设计目标:
#   - 单一脚本完成 上传 → 备份 → 替换 → 重启 → 健康检查 → 回滚提示
#   - 配置/数据/Agent 全部不动,只换二进制
#   - 失败安全:任意步骤出错都打印回滚命令,且备份始终保留
#
# 用法:
#   ./scripts/deploy-update.sh root@your-server-ip
#   REMOTE_HOST=root@your-server-ip ./scripts/deploy-update.sh
#
# 环境变量(可选):
#   REMOTE_PORT   SSH 端口,默认 22
#   SSH_KEY       SSH 私钥路径(强烈建议提前 ssh-copy-id 配好),默认走 ssh-agent / ~/.ssh/id_*
#   SSH_PASS      远端密码(用 sshpass 自动化,仅在没有 SSH key 时使用);
#                 强烈建议尽快配 key 后清除此变量。需 brew install hudochenkov/sshpass/sshpass。
#   TARGET        编译目标 triple,默认 x86_64-unknown-linux-musl
#   SERVICE_NAME  远端 systemd 服务名,默认 nodelite-server
#   BIN_PATH      远端二进制绝对路径,默认 /usr/local/bin/nodelite-server
#   HEALTH_URL    部署后 HTTP 健康探测 URL(可选,例如 http://your-server-ip:port/)
#   SKIP_BUILD=1  跳过本地编译,直接上传已有产物(用于失败后重试)
#   KEEP_BACKUPS  保留的旧二进制备份份数,默认 5

set -euo pipefail

# ----------------------------------------------------------------------------
# 参数解析
# ----------------------------------------------------------------------------
REMOTE_HOST="${1:-${REMOTE_HOST:-}}"
REMOTE_PORT="${REMOTE_PORT:-22}"
TARGET="${TARGET:-x86_64-unknown-linux-musl}"
SERVICE_NAME="${SERVICE_NAME:-nodelite-server}"
BIN_PATH="${BIN_PATH:-/usr/local/bin/nodelite-server}"
HEALTH_URL="${HEALTH_URL:-}"
KEEP_BACKUPS="${KEEP_BACKUPS:-5}"
SKIP_BUILD="${SKIP_BUILD:-0}"
SSH_KEY="${SSH_KEY:-}"
SSH_PASS="${SSH_PASS:-}"

if [[ -z "$REMOTE_HOST" ]]; then
  echo "ERROR: 缺少 REMOTE_HOST。" >&2
  echo "用法: $0 user@host  或  REMOTE_HOST=user@host $0" >&2
  exit 2
fi

# 组装 ssh / scp 通用选项
SSH_OPTS=(-p "$REMOTE_PORT" -o ServerAliveInterval=30 -o StrictHostKeyChecking=accept-new)
SCP_OPTS=(-P "$REMOTE_PORT" -o ServerAliveInterval=30 -o StrictHostKeyChecking=accept-new)
if [[ -n "$SSH_KEY" ]]; then
  SSH_OPTS+=(-i "$SSH_KEY")
  SCP_OPTS+=(-i "$SSH_KEY")
fi

# 若使用 SSH_PASS, 校验 sshpass 可用
if [[ -n "$SSH_PASS" ]]; then
  if ! command -v sshpass >/dev/null 2>&1; then
    echo "ERROR: 设置了 SSH_PASS 但未安装 sshpass。请运行:" >&2
    echo "  brew install hudochenkov/sshpass/sshpass" >&2
    exit 2
  fi
  # sshpass 默认拒绝在 TTY 上传密码 prompt, 这里强制走密码认证
  SSH_OPTS+=(-o PreferredAuthentications=password -o PubkeyAuthentication=no)
  SCP_OPTS+=(-o PreferredAuthentications=password -o PubkeyAuthentication=no)
fi

LOCAL_BIN="target/${TARGET}/release/nodelite-server"
TIMESTAMP="$(date +%Y%m%d-%H%M%S)"
REMOTE_TMP="/tmp/nodelite-server.new.${TIMESTAMP}"
REMOTE_BACKUP="${BIN_PATH}.bak.${TIMESTAMP}"

# ----------------------------------------------------------------------------
# 工具函数
# ----------------------------------------------------------------------------
log()   { printf '\033[1;34m[deploy]\033[0m %s\n' "$*"; }
ok()    { printf '\033[1;32m[ ok  ]\033[0m %s\n' "$*"; }
warn()  { printf '\033[1;33m[warn ]\033[0m %s\n' "$*" >&2; }
fail()  { printf '\033[1;31m[fail ]\033[0m %s\n' "$*" >&2; exit 1; }

ssh_remote() {
  if [[ -n "$SSH_PASS" ]]; then
    SSHPASS="$SSH_PASS" sshpass -e ssh "${SSH_OPTS[@]}" "$REMOTE_HOST" "$@"
  else
    ssh "${SSH_OPTS[@]}" "$REMOTE_HOST" "$@"
  fi
}

scp_to_remote() {
  if [[ -n "$SSH_PASS" ]]; then
    SSHPASS="$SSH_PASS" sshpass -e scp "${SCP_OPTS[@]}" "$@"
  else
    scp "${SCP_OPTS[@]}" "$@"
  fi
}

# 通过 ssh + cat 上传文件,绕过 OpenSSH 9.x 新版 scp(SFTP 后端)与 sshpass 的兼容性问题。
# 用法: upload_file <local_path> <remote_absolute_path>
upload_file() {
  local src="$1" dst="$2"
  if [[ -n "$SSH_PASS" ]]; then
    SSHPASS="$SSH_PASS" sshpass -e ssh "${SSH_OPTS[@]}" "$REMOTE_HOST" "cat > '$dst'" < "$src"
  else
    ssh "${SSH_OPTS[@]}" "$REMOTE_HOST" "cat > '$dst'" < "$src"
  fi
}

# ----------------------------------------------------------------------------
# 步骤 1: 本地编译(可跳过)
# ----------------------------------------------------------------------------
if [[ "$SKIP_BUILD" != "1" ]]; then
  log "本地交叉编译 → $TARGET (release)"
  if ! rustup target list --installed | grep -q "^${TARGET}$"; then
    log "添加 rustup target: $TARGET"
    rustup target add "$TARGET"
  fi
  cargo build --release --target "$TARGET" -p nodelite-server
  ok "编译完成"
else
  log "已设置 SKIP_BUILD=1,跳过编译"
fi

if [[ ! -x "$LOCAL_BIN" ]]; then
  fail "未找到产物: $LOCAL_BIN"
fi

LOCAL_SIZE="$(stat -f%z "$LOCAL_BIN" 2>/dev/null || stat -c%s "$LOCAL_BIN")"
LOCAL_SHA="$(shasum -a 256 "$LOCAL_BIN" | awk '{print $1}')"
log "产物大小: $((LOCAL_SIZE / 1024)) KB"
log "产物 SHA256: $LOCAL_SHA"

# ----------------------------------------------------------------------------
# 步骤 2: 预检远端环境
# ----------------------------------------------------------------------------
log "预检远端: $REMOTE_HOST (port $REMOTE_PORT)"
ssh_remote "test -x '$BIN_PATH' && systemctl is-enabled '$SERVICE_NAME.service' >/dev/null" \
  || fail "远端缺少现有 $BIN_PATH 或 $SERVICE_NAME.service 未启用。先用官方 install-server.sh 完成首次安装。"

REMOTE_ARCH="$(ssh_remote 'uname -m')"
[[ "$REMOTE_ARCH" == "x86_64" ]] || warn "远端架构 $REMOTE_ARCH 与本地编译目标 $TARGET 不一致!继续可能导致服务无法启动。"
ok "远端环境就绪 (arch=$REMOTE_ARCH)"

# ----------------------------------------------------------------------------
# 步骤 3: 上传到远端临时路径并校验
# ----------------------------------------------------------------------------
log "上传二进制 → ${REMOTE_HOST}:${REMOTE_TMP} (走 ssh cat 通道)"
upload_file "$LOCAL_BIN" "$REMOTE_TMP"

REMOTE_SHA="$(ssh_remote "sha256sum '$REMOTE_TMP' | awk '{print \$1}'")"
if [[ "$REMOTE_SHA" != "$LOCAL_SHA" ]]; then
  ssh_remote "rm -f '$REMOTE_TMP'" || true
  fail "SHA256 不一致(本地 $LOCAL_SHA / 远端 $REMOTE_SHA),已删除远端临时文件。"
fi
ok "上传完整性校验通过"

# ----------------------------------------------------------------------------
# 步骤 4: 原子替换 + 重启 + 备份保留策略
# ----------------------------------------------------------------------------
log "备份现有二进制 → $REMOTE_BACKUP"
ssh_remote "cp '$BIN_PATH' '$REMOTE_BACKUP' && chmod +x '$REMOTE_TMP' && mv '$REMOTE_TMP' '$BIN_PATH'"
ok "已替换 $BIN_PATH"

log "重启 $SERVICE_NAME.service"
ssh_remote "systemctl restart '$SERVICE_NAME.service'"

# 等待 systemd 转入 active 或失败
log "等待服务进入稳定状态(最多 15s)..."
DEADLINE=$((SECONDS + 15))
STATE="unknown"
while [[ $SECONDS -lt $DEADLINE ]]; do
  STATE="$(ssh_remote "systemctl is-active '$SERVICE_NAME.service' || true")"
  if [[ "$STATE" == "active" || "$STATE" == "failed" ]]; then
    break
  fi
  sleep 1
done

if [[ "$STATE" != "active" ]]; then
  warn "服务未达到 active(当前状态: $STATE),打印最近日志便于排查:"
  ssh_remote "journalctl -u '$SERVICE_NAME.service' -n 50 --no-pager" || true
  cat <<ROLLBACK >&2

──────────── 回滚命令(复制到本地粘贴执行) ────────────
ssh ${SSH_OPTS[*]} $REMOTE_HOST "
  set -e
  mv '$REMOTE_BACKUP' '$BIN_PATH'
  systemctl restart '$SERVICE_NAME.service'
  systemctl is-active '$SERVICE_NAME.service'
"
─────────────────────────────────────────────────────────
ROLLBACK
  exit 1
fi
ok "服务已 active"

# ----------------------------------------------------------------------------
# 步骤 5: 可选 HTTP 健康检查
# ----------------------------------------------------------------------------
if [[ -n "$HEALTH_URL" ]]; then
  log "HTTP 健康探测: $HEALTH_URL"
  HTTP_CODE="$(curl -s -o /dev/null -w '%{http_code}' --max-time 8 "$HEALTH_URL" || echo 000)"
  if [[ "$HTTP_CODE" =~ ^(2|3) ]]; then
    ok "HTTP $HTTP_CODE — 健康"
  else
    warn "HTTP $HTTP_CODE — 可能未启动完成,人工确认 $HEALTH_URL"
  fi
fi

# ----------------------------------------------------------------------------
# 步骤 6: 清理旧备份(只保留最近 $KEEP_BACKUPS 份)
# ----------------------------------------------------------------------------
log "清理超出 $KEEP_BACKUPS 份的旧备份"
ssh_remote "
  ls -1t '${BIN_PATH}.bak.'* 2>/dev/null | tail -n +$((KEEP_BACKUPS + 1)) | xargs -r rm -f
" || warn "清理旧备份失败(忽略)"

# ----------------------------------------------------------------------------
# 完成
# ----------------------------------------------------------------------------
cat <<DONE

╔══════════════════════════════════════════════════════════════╗
║                  ✅  部署完成                                 ║
╠══════════════════════════════════════════════════════════════╣
║  目标:        $REMOTE_HOST
║  二进制:      $BIN_PATH
║  当前备份:    $REMOTE_BACKUP
║  本地产物:    $LOCAL_BIN ($((LOCAL_SIZE / 1024)) KB)
║  SHA256:      $LOCAL_SHA
║  服务状态:    $STATE
╠══════════════════════════════════════════════════════════════╣
║  浏览器请硬刷新 (Cmd/Ctrl + Shift + R) 验证带鱼屏布局         ║
║                                                                ║
║  如需回滚:                                                     ║
║    ssh $REMOTE_HOST "                              ║
║      mv $REMOTE_BACKUP $BIN_PATH && ║
║      systemctl restart $SERVICE_NAME.service                  ║
║    "                                                           ║
╚══════════════════════════════════════════════════════════════╝
DONE
