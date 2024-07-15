

# Warmup
oha -n 100 $URL

REQUESTS=10000
PORT=8080
CONCURRENCY=1
URL="http://localhost:${PORT}"
reset
oha -n $REQUESTS -c $CONCURRENCY --latency-correction -disable-keepalive $URL