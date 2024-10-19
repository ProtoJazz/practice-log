import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import "@blueprintjs/core/lib/css/blueprint.css";
import "@blueprintjs/icons/lib/css/blueprint-icons.css";
import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { Alignment, Button, Navbar } from "@blueprintjs/core";
function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");
  const [regiments, setRegiments] = useState<string[]>([]);
  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name }));
  }
  async function createFullRegiment() {
    try {
      await invoke("create_full_regiment"); // Call the Rust function
      alert("Practice regiment created successfully!");
    } catch (error) {
      console.error("Error creating practice regiment:", error); // Log detailed error to the console
      alert(`Failed to create practice regiment: ${error}`);
    }
  }

  async function loadRegiments() {
    try {
      const loadedRegiments: string[] = await invoke("load_practice_regiments");
      setRegiments(loadedRegiments);
    } catch (error) {
      console.error("Failed to load regiments:", error);
    }
  }

  useEffect(() => {
    // Optionally load regiments when the component mounts
    loadRegiments();
  }, []);

  const [bpm, setBpm] = useState<number | null>(null);

  useEffect(() => {
    // Listen for MQTT BPM data from the backend
    const unlisten = listen<number>("mqtt_bpm", (event) => {
      setBpm(event.payload); // Set the received BPM value
    });

    // Clean up the event listener when the component unmounts
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  return (
    <main className="bp5-dark">
      <Navbar>
        <Navbar.Group align={Alignment.LEFT}>
          <Navbar.Heading>Blueprint</Navbar.Heading>
          <Navbar.Divider />
          <Button className="bp5-minimal" icon="home" text="Home" />
          <Button
            className="bp5-minimal"
            icon="document"
            text="Files"
            onClick={createFullRegiment}
          />
        </Navbar.Group>
      </Navbar>

      <div>
        <h1>MQTT BPM Reader</h1>
        <p>Received BPM: {bpm !== null ? bpm : "Waiting for data..."}</p>

        {regiments.length > 0 ? (
          <ul>
            {regiments.map((regiment, index) => (
              <li key={index}>{regiment}</li>
            ))}
          </ul>
        ) : (
          <p>Loading...</p>
        )}
      </div>
    </main>
  );
}

export default App;
