// Cheffers landing page — highlights the hero recipe with the Chef tokenizer
// shared with the playground and cookbook, and wires the "Copy" buttons on
// the install snippets. The theme toggle lives in ./assets/theme.js.
//
// No build step: this file is served as-is, as an ES module.

import { highlightChef } from "./assets/chef-syntax.js";

document.querySelectorAll("[data-chef]").forEach(highlightChef);

document.querySelectorAll("[data-copy]").forEach((button) => {
  const source = document.getElementById(button.dataset.copy);
  const label = button.querySelector(".copy-label");
  const icon = button.querySelector(".icon");
  let timer;

  const flash = (text, ok, ms) => {
    label.textContent = text;
    icon.classList.toggle("icon-check", ok);
    icon.classList.toggle("icon-copy", !ok);
    clearTimeout(timer);
    timer = setTimeout(() => {
      label.textContent = "Copy";
      icon.classList.replace("icon-check", "icon-copy");
    }, ms);
  };

  button.addEventListener("click", () => {
    navigator.clipboard.writeText(source.textContent.trim() + "\n").then(
      () => flash("Copied", true, 1400),
      () => flash("Press Ctrl+C", false, 1800),
    );
  });
});
