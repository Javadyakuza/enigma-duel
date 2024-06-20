import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter as Router, Route, Routes } from "react-router-dom";
import "./index.css";
import { EndMatch, Home, Match } from "./App";
import reportWebVitals from "./reportWebVitals";
import { Buffer } from "buffer";
import process from "process";

global.Buffer = Buffer;
global.process = process;

const root = ReactDOM.createRoot(document.getElementById("root"));
root.render(
  <React.StrictMode>
    <Router>
      <Routes>
        <Route path="/" element={<Home />} />
        <Route path="/match" element={<Match />} />
        <Route path="/endmatch" element={<EndMatch />} />
      </Routes>
    </Router>
  </React.StrictMode>
);

reportWebVitals();
