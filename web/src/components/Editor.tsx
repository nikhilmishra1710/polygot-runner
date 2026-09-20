import MonacoEditor from "@monaco-editor/react";

interface EditorProps {
  code: string;
  language: string;
  disabled: boolean;
  onChange: (value: string | undefined) => void;
  onLanguageChange: (language: string) => void;
}

export default function Editor({
  code,
  language,
  disabled,
  onChange,
  onLanguageChange,
}: EditorProps) {
  return (
    <div
      style={{
        flex: 1,
        display: "flex",
        flexDirection: "column",
        border: "1px solid #ccc",
      }}
    >
      <div
        style={{
          padding: "0.5rem",
          backgroundColor: "#f5f5f5",
          borderBottom: "1px solid #ccc",
        }}
      >
        <select
          value={language}
          onChange={(e) => onLanguageChange(e.target.value)}
          disabled={disabled}
          style={{ padding: "0.2rem 0.5rem" }}
        >
          <option value="python">Python</option>
          <option value="cpp">C++</option>
        </select>
      </div>

      <div style={{ flex: 1 }}>
        <MonacoEditor
          height="100%"
          language={language}
          theme="vs-dark"
          value={code}
          onChange={onChange}
          options={{
            readOnly: disabled,
            minimap: { enabled: false },
            scrollBeyondLastLine: false,
            fontSize: 14,
          }}
        />
      </div>
    </div>
  );
}
