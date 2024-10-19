import { Button, Card, FormGroup, InputGroup } from "@blueprintjs/core";
import { DateInput3 } from "@blueprintjs/datetime2";
import React, { useCallback, useState } from "react";
import { Regiment, PracticePieces } from "../types";
import moment from "moment";
import { invoke } from "@tauri-apps/api/core";
function CreatePracticeRegiment() {
  const [dateValue, setDateValue] = useState<string | null>(null);
  const [pieces, setPieces] = useState<PracticePieces[]>([]); // Store practice pieces
  const [newPieceName, setNewPieceName] = useState<string>(""); // For the input field of a new piece

  const handleDateChange = useCallback((date: string | null) => {
    setDateValue(date);
  }, []);

  // Handle the name of the new practice piece being added
  const handlePieceNameChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setNewPieceName(e.target.value);
  };
  const addPracticePiece = () => {
    if (newPieceName.trim() === "") return; // Avoid adding empty names
    const newPiece: PracticePieces = {
      id: Math.random().toString(36).substring(7), // Generate a random ID for the piece
      name: newPieceName,
    };
    setPieces((prevPieces) => [...prevPieces, newPiece]);
    setNewPieceName(""); // Clear the input field after adding
  };

  // Save the regiment
  const saveRegiment = async () => {
    const newRegiment: Regiment = {
      id: Math.random().toString(36).substring(7), // Generate a random ID for the regiment
      date: dateValue ? moment(dateValue) : moment().local(),
      pieces: pieces,
    };
    console.log("Saved Regiment:", newRegiment);
    try {
      await invoke("create_full_regiment", { regiment: newRegiment }); // Send the regiment to the backend
      console.log("Regiment saved successfully!");
    } catch (error) {
      console.error("Failed to save regiment:", error);
    }
    // Here, you would typically call a function to send this regiment to your backend
  };

  return (
    <div>
      <Card>
        <FormGroup
          helperText="Setup the pieces you want to practice in this regiment."
          label="New Regiment"
          labelFor="text-input"
          labelInfo="(required)"
        >
          <DateInput3
            onChange={handleDateChange}
            placeholder="M/D/YYYY"
            value={dateValue}
          />
          <InputGroup
            id="text-input"
            placeholder="Enter practice piece name"
            value={newPieceName}
            onChange={handlePieceNameChange}
          />
          {pieces.length > 0 && (
            <ul>
              {pieces.map((piece) => (
                <li key={piece.id}>{piece.name}</li>
              ))}
            </ul>
          )}
          <Button onClick={addPracticePiece}>Add Practice Piece</Button>
          <Button intent="success" onClick={saveRegiment}>
            Save Regiment
          </Button>
        </FormGroup>
      </Card>
    </div>
  );
}

export default CreatePracticeRegiment;
