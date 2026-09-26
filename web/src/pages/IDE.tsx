// src/App.tsx
import { useState } from "react";
import { useExecution } from "../hooks/useExecution";
import { type Language, SUPPORTED_LANGUAGES } from "../types/language";

import Header from "../components/Header";
import Editor from "../components/Editor";
import OutputPanel from "../components/OutputPanel";
import RunButton from "../components/RunButton";
import StatusBar from "../components/StatusBar";

export default function IDE() {
  const [selectedLanguage, setSelectedLanguage] = useState<Language>(
    SUPPORTED_LANGUAGES[0],
  );
  const [code, setCode] = useState(selectedLanguage.starter_boilerplate);

  const { status, stdout, stderr, run, cancel } = useExecution();

  const handleLanguageChange = (lang: Language) => {
    setSelectedLanguage(lang);
    setCode(lang.starter_boilerplate);
  };

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        height: "100vh",
        fontFamily: "sans-serif",
      }}
    >
      <Header
        languages={SUPPORTED_LANGUAGES}
        selectedLanguage={selectedLanguage}
        onLanguageChange={handleLanguageChange}
        disabled={status === "RUNNING"}
      />

      {/* Main 50/50 Split Area */}
      <div style={{ display: "flex", flex: 1, overflow: "hidden" }}>
        {/* Editor Pane (Left 50%) */}
        <div style={{ flex: "1 1 50%", borderRight: "1px solid #ccc" }}>
          <Editor
            code={code}
            monacoLanguage={selectedLanguage.monacoLanguage}
            onChange={(val) => setCode(val || "")}
            disabled={status === "RUNNING"}
          />
        </div>

        {/* Output Pane (Right 50%) */}
        <div style={{ flex: "1 1 50%" }}>
          <OutputPanel stdout={stdout} stderr={stderr} status={status} />
        </div>
      </div>

      {/* Footer / Control Bar */}
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          padding: "1rem",
          backgroundColor: "#f8f9fa",
          borderTop: "1px solid #dee2e6",
        }}
      >
        <RunButton
          status={status}
          onRun={() => run(code, selectedLanguage)}
          onCancel={cancel}
        />
        <StatusBar status={status} />
      </div>
    </div>
  );
}
