// src/components/Editor.tsx
import MonacoEditor from "@monaco-editor/react";
import { type Language } from "../types/language";

interface EditorProps {
  code: string;
  onChange: (value: string) => void;
  languages: Language[];
  selectedLanguage: Language;
  onLanguageChange: (language: Language) => void;
  disabled?: boolean;
}

export default function Editor({
  code,
  onChange,
  languages,
  selectedLanguage,
  onLanguageChange,
  disabled = false,
}: EditorProps) {
  const handleLanguageSelect = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const lang = languages.find((l) => l.id === e.target.value);
    if (lang) {
      onLanguageChange(lang);
    }
  };

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
          borderBottom: "1px solid #ccc",
          backgroundColor: "#f5f5f5",
        }}
      >
        <select
          value={selectedLanguage.id}
          onChange={handleLanguageSelect}
          disabled={disabled}
          style={{ padding: "0.25rem", borderRadius: "4px" }}
        >
          {languages.map((lang) => (
            <option key={lang.id} value={lang.id}>
              {lang.name}
            </option>
          ))}
        </select>
      </div>

      <div style={{ flex: 1 }}>
        <MonacoEditor
          language={selectedLanguage.monacoLanguage}
          value={code}
          onChange={(value) => onChange(value || "")}
          options={{
            minimap: { enabled: false },
            fontSize: 14,
            readOnly: disabled,
          }}
        />
      </div>
    </div>
  );
}
