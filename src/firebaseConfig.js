// Your Firebase project's settings. Paste them here once (see docs/SYNC_AND_MOBILE.md, step 3):
// Firebase console → Project settings → General → Your apps → the web app (</>) → "SDK setup and configuration" → Config.
// These values are not secret. What protects your family's data is firestore.rules, not hiding this file.
export const firebaseConfig = {
  apiKey: "AIzaSyDs0NTTshg738fFi5z-YRBmxS3cc9NZvmk",
  authDomain: "the-vault-1399a.firebaseapp.com",
  projectId: "the-vault-1399a",
  storageBucket: "the-vault-1399a.firebasestorage.app",
  messagingSenderId: "302163716121",
  appId: "1:302163716121:web:682ad276b31f9edc46a288",
};

export const isFirebaseConfigured = !firebaseConfig.apiKey.startsWith("PASTE");
