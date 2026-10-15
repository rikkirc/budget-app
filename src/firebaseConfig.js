// Your Firebase project's settings. Paste them here once (see docs/SYNC_AND_MOBILE.md, step 3):
// Firebase console → Project settings → General → Your apps → the web app (</>) → "SDK setup and configuration" → Config.
// These values are not secret. What protects your family's data is firestore.rules, not hiding this file.
export const firebaseConfig = {
  apiKey: "PASTE-YOUR-API-KEY",
  authDomain: "your-project.firebaseapp.com",
  projectId: "your-project",
  storageBucket: "your-project.firebasestorage.app",
  messagingSenderId: "",
  appId: "",
};

export const isFirebaseConfigured = !firebaseConfig.apiKey.startsWith("PASTE");
