

# Warmup
oha -n 100 $URL

REQUESTS=1
PORT=8080
CONCURRENCY=50
URL="http://localhost:${PORT}"
reset
oha -n $REQUESTS -c $CONCURRENCY --latency-correction -d foo -m POST $URL