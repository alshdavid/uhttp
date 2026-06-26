const root = document.querySelector('main')

switch (window.location.pathname) {
  case "/":
    // SPA-ish, use a router
    window.location.assign("/home");
    break;
  case "/home":
    root.innerHTML = `
      <h1>Home Page</h1>
    `
    break;
  case "/about":
    root.innerHTML = `
      <h1>About Page</h1>
    `
    break;
  default:
    root.innerHTML = `
      <h1>Not Found</h1>
      <h2>${window.location.pathname}</h2>
    `
}
