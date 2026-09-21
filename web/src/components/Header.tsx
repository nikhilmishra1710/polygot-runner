// src/components/Header.tsx
import { type Language } from "../types/language";

interface HeaderProps {
  languages: Language[];
  selectedLanguage: Language;
  onLanguageChange: (language: Language) => void;
  disabled?: boolean;
}

export default function Header({
  languages,
  selectedLanguage,
  onLanguageChange,
  disabled = false,
}: HeaderProps) {
  return (
    <header
      style={{
        display: "flex",
        justifyContent: "space-between",
        alignItems: "center",
        padding: "1rem",
        backgroundColor: "#f8f9fa",
        borderBottom: "1px solid #dee2e6",
      }}
    >
      <h1 style={{ margin: 0, fontSize: "1.25rem", fontWeight: 600 }}>
        Polyglot Runtime
      </h1>
      <select
        value={selectedLanguage.id}
        onChange={(e) => {
          const lang = languages.find((l) => l.id === e.target.value);
          if (lang) onLanguageChange(lang);
        }}
        disabled={disabled}
        style={{ padding: "0.4rem", borderRadius: "4px" }}
      >
        {languages.map((lang) => (
          <option key={lang.id} value={lang.id}>
            {lang.name}
          </option>
        ))}
      </select>
    </header>
  );
}
