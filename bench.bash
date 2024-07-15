

# Warmup
oha -n 1000 $URL

REQUESTS=10000
PORT=8080
URL="http://localhost:${PORT}"
reset
oha -n $REQUESTS -c 1000 --latency-correction -disable-keepalive $URL