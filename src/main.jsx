import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import UpdateBanner from "./UpdateBanner";
import { IS_WEB } from "./api";
import "./mobile.css";

ReactDOM.createRoot(document.getElementById("root")).render(
  <React.StrictMode>
    <App />
    {IS_WEB && <UpdateBanner />}
  </React.StrictMode>,
);
