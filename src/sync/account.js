// Family accounts on Firebase Authentication. Anyone with the family invite code can create their own account;
// firestore.rules only lets accounts that joined with a valid code store anything.
import {
  createUserWithEmailAndPassword, EmailAuthProvider, reauthenticateWithCredential, sendPasswordResetEmail,
  signInWithEmailAndPassword, signOut as firebaseSignOut, updatePassword,
} from "firebase/auth";
import { doc, getDoc, serverTimestamp, setDoc } from "firebase/firestore";

export const MIN_PASSWORD_LEN = 8;

// Profile names become vault keys, so only allow the characters a person would type for a name
export function vaultKey(profile) {
  const p = profile.trim();
  if (!p || p.length > 64 || !/^[\p{L}\p{N} _-]+$/u.test(p)) {
    throw "ERROR: To sync, profile names may only use letters, numbers, spaces, - and _";
  }
  return p.toLowerCase().replaceAll(" ", "_");
}

// Firebase's error codes, in words a family member understands
export function explain(e) {
  const messages = {
    "auth/invalid-credential": "Wrong email or password.",
    "auth/wrong-password": "Wrong email or password.",
    "auth/user-not-found": "Wrong email or password.",
    "auth/email-already-in-use": "There is already an account with that email. Sign in instead.",
    "auth/invalid-email": "Please enter a valid email address.",
    "auth/weak-password": `Your password needs at least ${MIN_PASSWORD_LEN} characters.`,
    "auth/too-many-requests": "Too many attempts. Please wait a few minutes and try again.",
    "auth/network-request-failed": "Couldn't reach the sync service. Check your internet connection.",
    "auth/requires-recent-login": "Please sign out and back in, then try again.",
  };
  return messages[e?.code] ?? e?.message ?? String(e);
}

const fail = (e) => { throw Object.assign(new Error(explain(e)), { code: e?.code }); };

export async function signUp({ auth, db }, email, password, inviteCode) {
  if (password.length < MIN_PASSWORD_LEN) fail({ code: "auth/weak-password" });
  if (!inviteCode.trim()) fail({ message: "Please enter the family invite code." });
  const cred = await createUserWithEmailAndPassword(auth, email.trim(), password).catch(fail);
  try {
    await setDoc(doc(db, "members", cred.user.uid), { invite: inviteCode.trim(), email: cred.user.email, joinedAt: serverTimestamp() });
  } catch {
    // The invite code was wrong: remove the half-made account so the email can be used again
    await cred.user.delete().catch(() => firebaseSignOut(auth));
    fail({ message: "That family invite code isn't right. Ask whoever set up Vault's sync." });
  }
  return cred.user;
}

export async function signIn({ auth, db }, email, password) {
  const cred = await signInWithEmailAndPassword(auth, email.trim(), password).catch(fail);
  const member = await getDoc(doc(db, "members", cred.user.uid)).catch(() => null);
  if (member && !member.exists()) {
    await firebaseSignOut(auth);
    fail({ message: "This account never finished joining. Create it again with the family invite code." });
  }
  return cred.user;
}

/** Emails a password-reset link (Firebase sends it; it's free) */
export const sendReset = ({ auth }, email) => sendPasswordResetEmail(auth, email.trim()).catch(fail);

/** Changing the password also signs out every other device (Firebase ends their sessions) */
export async function changePassword({ auth }, currentPassword, newPassword) {
  if (newPassword.length < MIN_PASSWORD_LEN) fail({ code: "auth/weak-password" });
  const user = auth.currentUser;
  await reauthenticateWithCredential(user, EmailAuthProvider.credential(user.email, currentPassword)).catch(fail);
  await updatePassword(user, newPassword).catch(fail);
}

export const signOut = ({ auth }) => firebaseSignOut(auth);
