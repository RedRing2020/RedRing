#!/usr/bin/env bash
set -euo pipefail

JOB_TYPE=""
INPUT_PATH=""
OUTPUT_DIR="/work/output"
LOG_DIR="/work/logs"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --job-type)
      JOB_TYPE="$2"
      shift 2
      ;;
    --input)
      INPUT_PATH="$2"
      shift 2
      ;;
    --output)
      OUTPUT_DIR="$2"
      shift 2
      ;;
    --logs)
      LOG_DIR="$2"
      shift 2
      ;;
    *)
      echo "Unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

if [[ -z "$JOB_TYPE" ]]; then
  echo "--job-type is required (cam|sim)" >&2
  exit 2
fi

if [[ -z "$INPUT_PATH" ]]; then
  echo "--input is required" >&2
  exit 2
fi

if [[ "$JOB_TYPE" != "cam" && "$JOB_TYPE" != "sim" ]]; then
  echo "Unsupported --job-type: $JOB_TYPE" >&2
  exit 2
fi

if [[ ! -f "$INPUT_PATH" ]]; then
  echo "Input not found: $INPUT_PATH" >&2
  exit 3
fi

mkdir -p "$OUTPUT_DIR" "$LOG_DIR"

STARTED_AT="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
OUT_FILE="$OUTPUT_DIR/${JOB_TYPE}_result.json"
LOG_FILE="$LOG_DIR/${JOB_TYPE}.log"

{
  echo "[$STARTED_AT] start job_type=$JOB_TYPE input=$INPUT_PATH"
  echo "uid=$(id -u) gid=$(id -g)"
} > "$LOG_FILE"

cat > "$OUT_FILE" <<EOF
{
  "job_type": "$JOB_TYPE",
  "input": "$INPUT_PATH",
  "status": "succeeded",
  "runner": "redring-batch-runner-stub",
  "executed_at_utc": "$STARTED_AT"
}
EOF

echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] completed output=$OUT_FILE" >> "$LOG_FILE"

echo "completed: $OUT_FILE"
