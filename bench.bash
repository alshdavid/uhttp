RATE="25000"
NAME="actix"

echo "GET http://localhost:8080" | vegeta attack -rate="${RATE}" -duration=5s \
  -output=results.bin && cat results.bin | \
  vegeta plot > results/${RATE}_${NAME}.html && \
  rm results.bin

echo "<code><pre>" >> results/${RATE}_${NAME}.html
echo "GET http://localhost:8080" | vegeta attack -rate="${RATE}" -duration=5s | vegeta report --type=text >> results/${RATE}_${NAME}.html
echo "</pre></code>" >> results/${RATE}_${NAME}.html
