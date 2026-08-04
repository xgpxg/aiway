#!/bin/bash

# ============================================================
# AIWay Gateway 持续性压力测试脚本
# 依赖: wrk (https://github.com/wg/wrk)
# 用法: ./bench.sh [OPTIONS]
# ============================================================

set -euo pipefail

# -------------------- 默认配置 --------------------
TARGET_URL="http://127.0.0.1:7001/api/hello"
DURATION="60s"
THREADS_LIST=(2 4 8 16 32)
CONNECTIONS_LIST=(10 50 100 200 500)
RESULT_DIR="./bench-results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
SUMMARY_FILE=""

# -------------------- 颜色定义 --------------------
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

# -------------------- 参数解析 --------------------
usage() {
    cat <<EOF
用法: $0 [OPTIONS]

选项:
  -u, --url <URL>          目标 URL (默认: http://127.0.0.1:7080/)
  -d, --duration <TIME>    每轮持续时间 (默认: 60s)
  -t, --threads <LIST>     线程数列表，逗号分隔 (默认: 2,4,8,16,32)
  -c, --connections <LIST> 连接数列表，逗号分隔 (默认: 10,50,100,200,500)
  -o, --output <DIR>       结果输出目录 (默认: ./bench-results)
  -h, --help               显示帮助
EOF
    exit 0
}

while [[ $# -gt 0 ]]; do
    case $1 in
        -u|--url) TARGET_URL="$2"; shift 2 ;;
        -d|--duration) DURATION="$2"; shift 2 ;;
        -t|--threads) IFS=',' read -ra THREADS_LIST <<< "$2"; shift 2 ;;
        -c|--connections) IFS=',' read -ra CONNECTIONS_LIST <<< "$2"; shift 2 ;;
        -o|--output) RESULT_DIR="$2"; shift 2 ;;
        -h|--help) usage ;;
        *) echo "未知参数: $1"; usage ;;
    esac
done

SUMMARY_FILE="${RESULT_DIR}/summary_${TIMESTAMP}.txt"

# -------------------- 前置检查 --------------------
check_prerequisites() {
    if ! command -v wrk &>/dev/null; then
        echo -e "${RED}错误: wrk 未安装${NC}"
        echo "安装方式: sudo apt install wrk 或从 https://github.com/wg/wrk 编译"
        exit 1
    fi
    echo -e "${CYAN}目标地址:${NC} ${TARGET_URL}"
    echo -e "${CYAN}每轮时长:${NC} ${DURATION}"
    echo -e "${CYAN}线程列表:${NC} ${THREADS_LIST[*]}"
    echo -e "${CYAN}连接列表:${NC} ${CONNECTIONS_LIST[*]}"
    echo -e "${CYAN}结果目录:${NC} ${RESULT_DIR}"
    echo ""

    if ! curl -s -o /dev/null -w '' --connect-timeout 3 "${TARGET_URL}" 2>/dev/null; then
        echo -e "${YELLOW}警告: 目标 ${TARGET_URL} 可能不可达，继续压测...${NC}"
        echo ""
    fi
}

# -------------------- wrk 结果解析 --------------------
parse_wrk_output() {
    local output="$1"

    local requests_per_sec=$(echo "$output" | grep -oP 'Requests/sec:\s+\K[\d.]+' || echo "0")
    local transfer_per_sec=$(echo "$output" | grep -oP 'Transfer/sec:\s+\K[\d.]+\s*\w+' || echo "N/A")
    local total_requests=$(echo "$output" | grep -oP '\d+\s+requests in ' | grep -oP '^\d+' || echo "0")
    local errors_connect=$(echo "$output" | grep -oP 'Connect\s+\K\d+' || echo "0")
    local errors_read=$(echo "$output" | grep -oP 'Read\s+\K\d+' || echo "0")
    local errors_write=$(echo "$output" | grep -oP 'Write\s+\K\d+' || echo "0")
    local errors_timeout=$(echo "$output" | grep -oP 'Timeout\s+\K\d+' || echo "0")
    # Latency 行格式: "    Latency     4.12ms   22.06ms 208.97ms   97.02%"
    # 第2列=Avg, 第3列=Stdev, 第4列=Max
    local latency_line=$(echo "$output" | grep -P '^\s+Latency\s' | grep -v 'Distribution' || echo "")
    local lat_avg=$(echo "$latency_line" | awk '{print $2}')
    local lat_max=$(echo "$latency_line" | awk '{print $4}')
    [[ -z "$lat_avg" ]] && lat_avg="N/A"
    [[ -z "$lat_max" ]] && lat_max="N/A"
    local latency_p50=$(echo "$output" | grep -oP '50%\s+\K[\d.]+\w+' || echo "N/A")
    local latency_p99=$(echo "$output" | grep -oP '99%\s+\K[\d.]+\w+' || echo "N/A")

    echo "${requests_per_sec}|${transfer_per_sec}|${total_requests}|${errors_connect}|${errors_read}|${errors_write}|${errors_timeout}|${lat_avg}|${lat_max}|${latency_p50}|${latency_p99}"
}

# -------------------- 单轮压测 --------------------
run_single_bench() {
    local threads=$1
    local connections=$2
    local round_label="t${threads}_c${connections}"
    local raw_file="${RESULT_DIR}/${TIMESTAMP}_${round_label}.txt"

    echo -e "${YELLOW}▶ 线程=${threads}  连接=${connections}  时长=${DURATION}${NC}"

    local output
    output=$(wrk -t"${threads}" -c"${connections}" -d"${DURATION}" --latency "${TARGET_URL}" 2>&1) || true
    echo "$output" > "$raw_file"

    local parsed
    parsed=$(parse_wrk_output "$output")

    IFS='|' read -r rps transfer total_req err_conn err_read err_write err_timeout lat_avg lat_max lat_p50 lat_p99 <<< "$parsed"

    local total_errors=$((err_conn + err_read + err_write + err_timeout))

    echo -e "  ${GREEN}QPS:${NC} ${rps}  ${GREEN}吞吐:${NC} ${transfer}  ${GREEN}总请求:${NC} ${total_req}"
    echo -e "  ${GREEN}延迟 Avg:${NC} ${lat_avg}  ${GREEN}P50:${NC} ${lat_p50}  ${GREEN}P99:${NC} ${lat_p99}  ${GREEN}Max:${NC} ${lat_max}"
    if [[ ${total_errors} -gt 0 ]]; then
        echo -e "  ${RED}错误: 连接=${err_conn} 读取=${err_read} 写入=${err_write} 超时=${err_timeout}${NC}"
    else
        echo -e "  ${GREEN}错误: 0${NC}"
    fi
    echo ""

    echo "${threads}|${connections}|${rps}|${transfer}|${total_req}|${err_conn}|${err_read}|${err_write}|${err_timeout}|${lat_avg}|${lat_max}|${lat_p50}|${lat_p99}" >> "$SUMMARY_FILE"
}

# -------------------- 汇总报告 --------------------
print_summary() {
    echo ""
    echo -e "${BOLD}======================================== 汇总报告 ========================================${NC}"
    echo ""
    printf "${BOLD}%-10s %-12s %-12s %-16s %-12s %-12s %-12s %-12s %-12s %-14s %-12s %-12s %-12s${NC}\n" \
        "线程" "连接" "QPS" "吞吐/s" "总请求" "连接错" "读错" "写错" "超时" "Avg延迟" "P50" "P99" "Max"
    printf '%.0s─' $(seq 1 160); echo ""

    while IFS='|' read -r t c rps transfer total_req err_conn err_read err_write err_timeout lat_avg lat_max lat_p50 lat_p99; do
        printf "%-8s %-10s %-12s %-14s %-10s %-10s %-10s %-10s %-10s %-12s %-12s %-12s %-12s\n" \
            "$t" "$c" "$rps" "$transfer" "$total_req" "$err_conn" "$err_read" "$err_write" "$err_timeout" "$lat_avg" "$lat_p50" "$lat_p99" "$lat_max"
    done < "$SUMMARY_FILE"

    echo ""
    echo -e "${GREEN}详细结果已保存至: ${RESULT_DIR}/${TIMESTAMP}_*.txt${NC}"
    echo -e "${GREEN}汇总数据文件: ${SUMMARY_FILE}${NC}"
}

# -------------------- 主流程 --------------------
main() {
    check_prerequisites
    mkdir -p "$RESULT_DIR"

    echo -e "${BOLD}======================================== 开始压测 ========================================${NC}"
    echo ""

    > "$SUMMARY_FILE"

    local total_rounds=$(( ${#THREADS_LIST[@]} * ${#CONNECTIONS_LIST[@]} ))
    local current=0

    for t in "${THREADS_LIST[@]}"; do
        for c in "${CONNECTIONS_LIST[@]}"; do
            current=$((current + 1))
            echo -e "${CYAN}[${current}/${total_rounds}]${NC}"
            run_single_bench "$t" "$c"
        done
    done

    print_summary
}

main "$@"
