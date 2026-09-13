import React from "react";
import { createRoot } from "react-dom/client";
import "./styles.css";
import RootApp from "./RootApp";

const rootElement = document.getElementById("root");
if (!rootElement) {
  throw new Error("Missing #root mount point");
}

createRoot(rootElement).render(
  <React.StrictMode>
    <RootApp />
  </React.StrictMode>,
);
