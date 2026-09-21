// src/App.tsx
import { useState } from "react";
import { useExecution } from "./hooks/useExecution";
import { type Language, SUPPORTED_LANGUAGES } from "./types/language";

import Editor from "./components/Editor";
import OutputPanel from "./components/OutputPanel";
import RunButton from "./components/RunButton";
import StatusBar from "./components/StatusBar";

export default function App() {
  const [selectedLanguage, setSelectedLanguage] = useState<Language>(
    SUPPORTED_LANGUAGES[0],
  );
  const [code, setCode] = useState(selectedLanguage.starter_boilerplate);

  // Consume our new hook
  const { status, output, run, cancel } = useExecution();

  const updateLanguage = (lang: Language) => {
    setSelectedLanguage(lang);
    setCode(lang.starter_boilerplate);
  };

  const handleRun = () => {
    run(code, selectedLanguage);
  };

  return (
    <div
      style={{
        padding: "2rem",
        maxWidth: "1200px",
        margin: "0 auto",
        fontFamily: "sans-serif",
      }}
    >
      <h2 style={{ marginBottom: "1rem" }}>Polyglot Runtime</h2>

      <div
        style={{
          display: "flex",
          gap: "1rem",
          height: "500px",
          marginBottom: "1rem",
        }}
      >
        <Editor
          code={code}
          languages={SUPPORTED_LANGUAGES}
          selectedLanguage={selectedLanguage}
          disabled={status === "RUNNING"}
          onChange={(val) => setCode(val || "")}
          onLanguageChange={updateLanguage}
        />
        <OutputPanel output={output} />
      </div>

      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
        }}
      >
        <RunButton status={status} onRun={handleRun} onCancel={cancel} />
        <StatusBar status={status} />
      </div>
    </div>
  );
}
