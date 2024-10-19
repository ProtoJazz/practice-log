import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Regiment } from "../types";
import {
  Button,
  Card,
  Elevation,
  Section,
  SectionCard,
} from "@blueprintjs/core";
import moment from "moment";

function PracticeRegiments() {
  const [regiments, setRegiments] = useState<Regiment[]>([]);
  const [error, setError] = useState<string | null>(null);

  const regimentReviver = (key: string, value: any) => {
    if (key === "date") {
      return moment(value);
    }
    return value;
  };

  async function loadRegiments() {
    try {
      const loadedRegiments: string = await invoke("load_practice_regiments");
      let parsedRegiments = JSON.parse(loadedRegiments, regimentReviver);
      setRegiments(parsedRegiments);
    } catch (error) {
      console.error("Failed to load regiments:", error);
    }
  }

  useEffect(() => {
    loadRegiments();
  }, []);

  return (
    <div>
      <h1>Practice Regiments</h1>
      {error && <p>{error}</p>}
      {regiments.length > 0 ? (
        <div>
          {regiments.map((regiment, index) => (
            <Section
              title={`Practice week of : ${regiment.date.format("MMMM Do")}`}
              className="regiment-section"
            >
              {regiment.pieces.map((piece, index) => (
                <SectionCard padded>{piece.name}</SectionCard>
              ))}
            </Section>
          ))}
        </div>
      ) : (
        <p>Loading...</p>
      )}
    </div>
  );
}

export default PracticeRegiments;
