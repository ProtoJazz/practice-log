import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { PracticePieceLogs, Regiment } from "../types";
import {
  Button,
  Card,
  Elevation,
  Section,
  SectionCard,
} from "@blueprintjs/core";
import moment from "moment";
import { Sparklines, SparklinesLine } from "react-sparklines";

function PracticeRegiments() {
  const [regiments, setRegiments] = useState<Regiment[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [activePiece, setActivePiece] = useState<number | null>(null);

  const regimentReviver = (key: string, value: any) => {
    if (key === "date" || key === "timestamp") {
      return moment(value);
    }
    return value;
  };
  type LogSparkProps = {
    logs: PracticePieceLogs[];
  };
  const LogSpark = (props: LogSparkProps) => {
    const bpmValues = props.logs.map((log) => log.bpm);
    const maxBpm = Math.max(...bpmValues);

    return (
      <div>
        <span>Max BPM: {maxBpm}</span>
        <div className="log-graph">
          <Sparklines data={bpmValues}>
            <SparklinesLine color="blue" />
          </Sparklines>
        </div>
      </div>
    );
  };

  async function loadRegiments() {
    try {
      const loadedRegiments: string = await invoke("load_practice_regiments_2");
      let parsedRegiments = JSON.parse(loadedRegiments, regimentReviver);
      console.log("Loaded regiments:", parsedRegiments);
      setRegiments(parsedRegiments);
    } catch (error) {
      console.error("Failed to load regiments:", error);
    }
  }

  const getActivePiece = async () => {
    try {
      const activePieceId = await invoke("get_active_piece");
      console.log("Active practice piece ID:", activePieceId);
      return activePieceId;
    } catch (error) {
      console.error("Failed to retrieve active practice piece:", error);
    }
  };

  const markPieceAsActive = async (practicePieceId: number | undefined) => {
    if (practicePieceId === undefined) {
      console.error("Invalid practice piece ID:", practicePieceId);
      return;
    }
    console.log("Marking practice piece as active:", practicePieceId);
    try {
      await invoke("mark_active_piece", { practicePieceId: practicePieceId });
      console.log("Practice piece marked as active!");
      setActivePiece(practicePieceId);
    } catch (error) {
      console.error("Failed to mark practice piece as active:", error);
    }
  };

  useEffect(() => {
    loadRegiments();
  }, []);

  useEffect(() => {
    getActivePiece();
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
                <>
                  <SectionCard padded>
                    {piece.name}
                    <Button
                      onClick={() => {
                        markPieceAsActive(piece.id);
                      }}
                      disabled={activePiece === piece.id}
                    >
                      Mark as Active
                    </Button>
                  </SectionCard>
                  <SectionCard padded>
                    <LogSpark logs={piece.logs} />
                  </SectionCard>
                </>
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
