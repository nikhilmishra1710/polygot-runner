// web/src/types/language.ts

export interface Language {
  id: string; // The identifier expected by the Go/Rust backend
  name: string; // Display name for the UI
  monacoLanguage: string; // The identifier expected by the Monaco Editor
  fileName: string; // Default filename for the source file payload
  starter_boilerplate: string;
}

// Only include languages currently supported by the Rust sandbox
export const SUPPORTED_LANGUAGES: Language[] = [
  {
    id: "python",
    name: "Python",
    monacoLanguage: "python",
    fileName: "main.py",
    starter_boilerplate: 'print("Hello from runner")',
  },
  {
    id: "cpp",
    name: "C++",
    monacoLanguage: "cpp",
    fileName: "main.cpp",
    starter_boilerplate:
      '#include <iostream>\nusing namespace std;\nint main() {\n    cout << "Hello from runner" << endl;\n    return 0;\n}',
  },
];
