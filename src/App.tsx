import React from "react";
import { BrowserRouter as Router, Route, Routes } from "react-router-dom";
import AppNavbar from "./components/AppNavbar"; // Import your Navbar component
import Home from "./pages/Home";
import About from "./pages/About";
import PracticeRegiments from "./pages/PracticeRegiments";
import "./App.css";
import "@blueprintjs/core/lib/css/blueprint.css";
import "@blueprintjs/icons/lib/css/blueprint-icons.css";
import CreatePracticeRegiment from "./pages/CreatePracticeRegiment";

function App() {
  return (
    <Router>
      <main className="bp5-dark">
        <AppNavbar />
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="/about" element={<About />} />
          <Route path="/practice-regiments" element={<PracticeRegiments />} />
          <Route
            path="/create-practice-regiment"
            element={<CreatePracticeRegiment />}
          />
        </Routes>
      </main>
    </Router>
  );
}

export default App;
