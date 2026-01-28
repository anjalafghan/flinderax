#!/usr/bin/env bash

URL="http://0.0.0.0:3000"
DURATION="30s"
THREADS=4
SCRIPT="get_all_cards.lua"

# Concurrency ladder
CONNECTIONS=(10 50 100 250 500 750 1000)

echo "🚀 Axum Load Test: /card/get_all_cards"
echo "--------------------------------------"

for C in "${CONNECTIONS[@]}"; do
  echo ""
  echo "▶ Concurrency: $C"
  echo "--------------------------------------"

  OUTPUT=$(wrk -t$THREADS -c$C -d$DURATION --latency -s $SCRIPT $URL)

  RPS=$(echo "$OUTPUT" | grep "Requests/sec" | awk '{print $2}')
  AVG_LAT=$(echo "$OUTPUT" | grep "Latency" | head -1 | awk '{print $2}')
  P99_LAT=$(echo "$OUTPUT" | grep "99%" | awk '{print $2}')

  echo "Requests/sec        : $RPS"
  echo "Average Latency     : $AVG_LAT"
  echo "99th %ile Latency   : $P99_LAT"

  # Stop if avg latency > 100ms
  AVG_MS=$(echo "$AVG_LAT" | sed 's/ms//')
  if [[ "$AVG_MS" =~ ^[0-9.]+$ ]]; then
    if (( $(echo "$AVG_MS > 100" | bc -l) )); then
      echo "⚠️ Avg latency > 100ms — stopping test"
      break
    fi
  fi
done

echo ""
echo "✅ Stress test completed."

