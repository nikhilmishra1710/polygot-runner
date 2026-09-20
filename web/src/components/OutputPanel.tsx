interface OutputPanelProps {
  output: string;
}

export default function OutputPanel({ output }: OutputPanelProps) {
  return (
    <div
      style={{
        flex: 1,
        backgroundColor: "#1e1e1e",
        color: "#00ff00",
        padding: "1rem",
        fontFamily: "monospace",
        whiteSpace: "pre-wrap",
        overflowY: "auto",
      }}
    >
      {output || (
        <span style={{ color: "#666" }}>Output will appear here...</span>
      )}
    </div>
  );
}
