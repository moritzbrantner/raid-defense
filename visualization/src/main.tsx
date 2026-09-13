import React from "react";
import { createRoot } from "react-dom/client";
import "./styles.css";
import App from "./App";
import { GameWiki } from "./GameWiki";

const rootElement = document.getElementById("root");
if (!rootElement) {
  throw new Error("Missing #root mount point");
}

createRoot(rootElement).render(
  <React.StrictMode>
    <App />
    <GameWiki />
  </React.StrictMode>,
);
