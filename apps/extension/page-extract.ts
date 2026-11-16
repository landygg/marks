import { extractDocument, type Extracted } from "marks-extract";

// Injected into the tab by the background script, which reads the result of this call.
(globalThis as { marksExtract?: () => { page: Extracted; text: string } }).marksExtract = () => ({
  page: extractDocument(document.cloneNode(true) as Document, location.href),
  text: document.body?.innerText ?? "",
});
