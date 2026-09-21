// src/components/Editor.tsx
import MonacoEditor from "@monaco-editor/react";

interface EditorProps {
  code: string;
  monacoLanguage: string;
  onChange: (value: string) => void;
  disabled?: boolean;
}

export default function Editor({
  code,
  monacoLanguage,
  onChange,
  disabled = false,
}: EditorProps) {
  return (
    <div style={{ height: "100%", width: "100%" }}>
      <MonacoEditor
        language={monacoLanguage}
        value={code}
        onChange={(value) => onChange(value || "")}
        options={{
          minimap: { enabled: false },
          fontSize: 14,
          readOnly: disabled,
        }}
      />
    </div>
  );
}
