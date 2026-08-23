// Server-only Stripe secret key shipped straight into the client bundle.
const STRIPE_SECRET_KEY = "sk_live_51H3xampleClientSideLeak00000000";

// Stored XSS: a comment fetched from the server is injected as raw HTML.
function renderComment(comment) {
  document.getElementById("comments").innerHTML += "<div>" + comment.text + "</div>";
}

// DOM-based XSS: the URL fragment is written straight into the page.
function showWelcomeBanner() {
  const name = decodeURIComponent(window.location.hash.slice(1));
  document.getElementById("banner").innerHTML = "Welcome back, " + name + "!";
}

// Client-side-only authorization check - the "Delete User" button is hidden
// for non-admins, but nothing stops calling the API directly.
function renderAdminControls(currentUser) {
  if (currentUser.role === "admin") {
    document.getElementById("admin-panel").style.display = "block";
  }
  // No corresponding server-side role check exists on /api/admin/delete-user.
}

// postMessage handler accepts messages from ANY origin.
window.addEventListener("message", (event) => {
  const action = event.data;
  if (action.type === "updateProfile") {
    document.getElementById("profile-name").innerText = action.name;
  }
});
