import { LocalizationProvider } from "@moritzbrantner/i18n/react";
import React from "react";
import { createRoot } from "react-dom/client";
import "./styles.css";
import { createRaidDefenseI18n } from "./localization";
import RootApp from "./RootApp";
import { supportedLocales } from "./translations";

const rootElement = document.getElementById("root");
if (!rootElement) {
  throw new Error("Missing #root mount point");
}

const { i18n } = await createRaidDefenseI18n();

createRoot(rootElement).render(
  <React.StrictMode>
    <LocalizationProvider fallbackLocale="en" i18n={i18n} supportedLocales={supportedLocales}>
      <RootApp />
    </LocalizationProvider>
  </React.StrictMode>,
);
