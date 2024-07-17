

# Warmup
oha -n 100 $URL

REQUESTS=50000
PORT=8080
CONCURRENCY=50
URL="http://localhost:${PORT}"
reset
oha -n $REQUESTS -c $CONCURRENCY --latency-correction -disable-keepalive $URL