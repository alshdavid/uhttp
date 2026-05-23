void (async function main() {
  console.log("Fetching from API");
  let response = await fetch("/api");
  let body = await response.text();

  console.log("Got from API:", body);
  const div = document.createElement("div");
  div.innerHTML = body;
  document.body.appendChild(div);
})();
