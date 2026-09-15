import {
  assertTranslationResources,
  createBrowserI18n,
} from "@moritzbrantner/i18n";
import { supportedLocales, translations } from "./translations";

export function validateTranslations() {
  assertTranslationResources(translations, "en");
}

export async function createRaidDefenseI18n() {
  validateTranslations();
  return createBrowserI18n({
    fallbackLocale: "en",
    resources: translations,
    supportedLocales,
  });
}
