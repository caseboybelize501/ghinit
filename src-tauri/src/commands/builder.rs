use std::fs;
use std::process::Command;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub prompt: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectModule {
    pub name: String,
    pub description: String,
    pub files: Vec<GeneratedFile>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeneratedFile {
    pub path: String,    // relative path within project, e.g. "src/main.py"
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub language: String,
    pub tags: Vec<String>,
    pub structure: Vec<TemplateFile>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TemplateFile {
    pub path: String,
    pub content: String, // may contain {{placeholders}}
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BuildProjectRequest {
    pub project_name: String,
    pub description: String,
    pub template_id: Option<String>,
    pub freeform_prompt: Option<String>,
    pub generated_files: Vec<GeneratedFile>, // filled by AI before calling this
    pub private_repo: bool,
    pub output_dir: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BuildResult {
    pub success: bool,
    pub local_path: String,
    pub repo_url: Option<String>,
    pub message: String,
}

/// Returns the built-in template library
#[tauri::command]
pub async fn get_templates() -> Result<Vec<ProjectTemplate>, String> {
    Ok(vec![
        ProjectTemplate {
            id: "react-ts".into(),
            name: "React + TypeScript".into(),
            description: "Vite-powered React app with TypeScript".into(),
            language: "TypeScript".into(),
            tags: vec!["frontend".into(), "react".into(), "vite".into()],
            structure: vec![
                TemplateFile { 
                    path: "index.html".into(), 
                    content: "<!DOCTYPE html>\n<html lang=\"en\">\n  <head>\n    <meta charset=\"UTF-8\" />\n    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\" />\n    <title>{{project_name}}</title>\n  </head>\n  <body>\n    <div id=\"root\"></div>\n    <script type=\"module\" src=\"/src/main.tsx\"></script>\n  </body>\n</html>\n".into() 
                },
                TemplateFile { 
                    path: "src/App.tsx".into(), 
                    content: "import { useState } from 'react';\nimport './App.css';\n\n// {{description}}\nfunction App() {\n  const [count, setCount] = useState(0);\n\n  return (\n    <div className=\"app\">\n      <h1>{{project_name}}</h1>\n      <p>{{description}}</p>\n      <button onClick={() => setCount(c => c + 1)}>Count: {count}</button>\n    </div>\n  );\n}\n\nexport default App;\n".into() 
                },
                TemplateFile { 
                    path: "src/App.css".into(), 
                    content: ".app {\n  max-width: 800px;\n  margin: 2rem auto;\n  padding: 2rem;\n  text-align: center;\n  font-family: system-ui, -apple-system, sans-serif;\n}\n\nh1 {\n  color: #333;\n}\n\nbutton {\n  padding: 0.5rem 1rem;\n  font-size: 1rem;\n  cursor: pointer;\n  background: #007bff;\n  color: white;\n  border: none;\n  border-radius: 4px;\n}\n\nbutton:hover {\n  background: #0056b3;\n}\n".into() 
                },
                TemplateFile { 
                    path: "src/main.tsx".into(), 
                    content: "import React from 'react';\nimport ReactDOM from 'react-dom/client';\nimport App from './App';\nimport './index.css';\n\nReactDOM.createRoot(document.getElementById('root')!).render(\n  <React.StrictMode>\n    <App />\n  </React.StrictMode>\n);\n".into() 
                },
                TemplateFile { 
                    path: "src/index.css".into(), 
                    content: "* {\n  margin: 0;\n  padding: 0;\n  box-sizing: border-box;\n}\n\nbody {\n  font-family: system-ui, -apple-system, sans-serif;\n  background: #f5f5f5;\n}\n".into() 
                },
                TemplateFile { 
                    path: "package.json".into(), 
                    content: "{\n  \"name\": \"{{project_name}}\",\n  \"private\": true,\n  \"version\": \"0.1.0\",\n  \"type\": \"module\",\n  \"scripts\": {\n    \"dev\": \"vite\",\n    \"build\": \"tsc && vite build\",\n    \"preview\": \"vite preview\"\n  },\n  \"dependencies\": {\n    \"react\": \"^19.1.0\",\n    \"react-dom\": \"^19.1.0\"\n  },\n  \"devDependencies\": {\n    \"@types/react\": \"^19.1.8\",\n    \"@types/react-dom\": \"^19.1.6\",\n    \"@vitejs/plugin-react\": \"^4.6.0\",\n    \"typescript\": \"~5.8.3\",\n    \"vite\": \"^7.0.4\"\n  }\n}\n".into() 
                },
                TemplateFile { 
                    path: "tsconfig.json".into(), 
                    content: "{\n  \"compilerOptions\": {\n    \"target\": \"ES2020\",\n    \"useDefineForClassFields\": true,\n    \"lib\": [\"ES2020\", \"DOM\", \"DOM.Iterable\"],\n    \"module\": \"ESNext\",\n    \"skipLibCheck\": true,\n    \"moduleResolution\": \"bundler\",\n    \"allowImportingTsExtensions\": true,\n    \"resolveJsonModule\": true,\n    \"isolatedModules\": true,\n    \"noEmit\": true,\n    \"jsx\": \"react-jsx\",\n    \"strict\": true,\n    \"noUnusedLocals\": true,\n    \"noUnusedParameters\": true,\n    \"noFallthroughCasesInSwitch\": true\n  },\n  \"include\": [\"src\"],\n  \"references\": [{ \"path\": \"./tsconfig.node.json\" }]\n}\n".into() 
                },
                TemplateFile { 
                    path: "tsconfig.node.json".into(), 
                    content: "{\n  \"compilerOptions\": {\n    \"composite\": true,\n    \"skipLibCheck\": true,\n    \"module\": \"ESNext\",\n    \"moduleResolution\": \"bundler\",\n    \"allowSyntheticDefaultImports\": true\n  },\n  \"include\": [\"vite.config.ts\"]\n}\n".into() 
                },
                TemplateFile { 
                    path: "vite.config.ts".into(), 
                    content: "import { defineConfig } from 'vite';\nimport react from '@vitejs/plugin-react';\n\nexport default defineConfig({\n  plugins: [react()],\n});\n".into() 
                },
                TemplateFile { 
                    path: "README.md".into(), 
                    content: "# {{project_name}}\n\n{{description}}\n\n## Setup\n\n```bash\nnpm install\nnpm run dev\n```\n\n## Build\n\n```bash\nnpm run build\n```\n".into() 
                },
            ],
        },
        ProjectTemplate {
            id: "python-cli".into(),
            name: "Python CLI Tool".into(),
            description: "Argparse-based Python command line tool".into(),
            language: "Python".into(),
            tags: vec!["cli".into(), "python".into(), "tool".into()],
            structure: vec![
                TemplateFile { 
                    path: "main.py".into(), 
                    content: "#!/usr/bin/env python3\n\"\"\"{{description}}\"\"\"\nimport argparse\nimport sys\n\n\ndef main():\n    parser = argparse.ArgumentParser(\n        description='{{description}}'\n    )\n    parser.add_argument(\n        '--version',\n        action='version',\n        version='{{project_name}} 0.1.0'\n    )\n    parser.add_argument(\n        '-v', '--verbose',\n        action='store_true',\n        help='Enable verbose output'\n    )\n    \n    args = parser.parse_args()\n    \n    if args.verbose:\n        print(f\"Running {{project_name}} in verbose mode\")\n    \n    print(\"Hello from {{project_name}}!\")\n    return 0\n\n\nif __name__ == '__main__':\n    sys.exit(main())\n".into() 
                },
                TemplateFile { 
                    path: "requirements.txt".into(), 
                    content: "# Add your dependencies here\n# Example:\n# requests>=2.28.0\n".into() 
                },
                TemplateFile { 
                    path: "README.md".into(), 
                    content: "# {{project_name}}\n\n{{description}}\n\n## Installation\n\n```bash\npip install -r requirements.txt\n```\n\n## Usage\n\n```bash\npython main.py --help\npython main.py --verbose\n```\n\n## Development\n\n```bash\npython -m pytest  # if you add pytest to requirements.txt\n```\n".into() 
                },
                TemplateFile { 
                    path: ".gitignore".into(), 
                    content: "__pycache__/\n*.py[cod]\n*$py.class\n*.so\n.Python\n.env\n.venv\nvenv/\n*.egg-info/\ndist/\nbuild/\n.pytest_cache/\n*.log\n".into() 
                },
            ],
        },
        ProjectTemplate {
            id: "tauri-app".into(),
            name: "Tauri Desktop App".into(),
            description: "Cross-platform desktop app with Tauri + React".into(),
            language: "Rust/TypeScript".into(),
            tags: vec!["desktop".into(), "tauri".into(), "rust".into()],
            structure: vec![
                TemplateFile { 
                    path: "README.md".into(), 
                    content: "# {{project_name}}\n\n{{description}}\n\n## Development\n\n```bash\n# Install dependencies\nnpm install\n\n# Run in development mode\ncargo tauri dev\n```\n\n## Build\n\n```bash\ncargo tauri build\n```\n".into() 
                },
                TemplateFile { 
                    path: "package.json".into(), 
                    content: "{\n  \"name\": \"{{project_name}}\",\n  \"private\": true,\n  \"version\": \"0.1.0\",\n  \"type\": \"module\",\n  \"scripts\": {\n    \"dev\": \"vite\",\n    \"build\": \"tsc && vite build\",\n    \"preview\": \"vite preview\",\n    \"tauri\": \"tauri\"\n  },\n  \"dependencies\": {\n    \"react\": \"^19.1.0\",\n    \"react-dom\": \"^19.1.0\",\n    \"@tauri-apps/api\": \"^2\"\n  },\n  \"devDependencies\": {\n    \"@types/react\": \"^19.1.8\",\n    \"@types/react-dom\": \"^19.1.6\",\n    \"@vitejs/plugin-react\": \"^4.6.0\",\n    \"typescript\": \"~5.8.3\",\n    \"vite\": \"^7.0.4\",\n    \"@tauri-apps/cli\": \"^2\"\n  }\n}\n".into() 
                },
                TemplateFile { 
                    path: "index.html".into(), 
                    content: "<!DOCTYPE html>\n<html lang=\"en\">\n  <head>\n    <meta charset=\"UTF-8\" />\n    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\" />\n    <title>{{project_name}}</title>\n  </head>\n  <body>\n    <div id=\"root\"></div>\n    <script type=\"module\" src=\"/src/main.tsx\"></script>\n  </body>\n</html>\n".into() 
                },
                TemplateFile { 
                    path: "src/App.tsx".into(), 
                    content: "import { useState } from 'react';\nimport './App.css';\n\nfunction App() {\n  const [count, setCount] = useState(0);\n\n  return (\n    <div className=\"app\">\n      <h1>{{project_name}}</h1>\n      <p>Built with Tauri + React</p>\n      <button onClick={() => setCount(c => c + 1)}>\n        Count: {count}\n      </button>\n    </div>\n  );\n}\n\nexport default App;\n".into() 
                },
                TemplateFile { 
                    path: "src/App.css".into(), 
                    content: ".app {\n  max-width: 800px;\n  margin: 2rem auto;\n  padding: 2rem;\n  text-align: center;\n  font-family: system-ui, -apple-system, sans-serif;\n}\n\nh1 {\n  color: #333;\n}\n\nbutton {\n  padding: 0.5rem 1rem;\n  font-size: 1rem;\n  cursor: pointer;\n  background: #007bff;\n  color: white;\n  border: none;\n  border-radius: 4px;\n}\n\nbutton:hover {\n  background: #0056b3;\n}\n".into() 
                },
                TemplateFile { 
                    path: "src/main.tsx".into(), 
                    content: "import React from 'react';\nimport ReactDOM from 'react-dom/client';\nimport App from './App';\nimport './index.css';\n\nReactDOM.createRoot(document.getElementById('root')!).render(\n  <React.StrictMode>\n    <App />\n  </React.StrictMode>\n);\n".into() 
                },
                TemplateFile { 
                    path: "src/index.css".into(), 
                    content: "* {\n  margin: 0;\n  padding: 0;\n  box-sizing: border-box;\n}\n\nbody {\n  font-family: system-ui, -apple-system, sans-serif;\n  background: #f5f5f5;\n}\n".into() 
                },
                TemplateFile { 
                    path: "tsconfig.json".into(), 
                    content: "{\n  \"compilerOptions\": {\n    \"target\": \"ES2020\",\n    \"useDefineForClassFields\": true,\n    \"lib\": [\"ES2020\", \"DOM\", \"DOM.Iterable\"],\n    \"module\": \"ESNext\",\n    \"skipLibCheck\": true,\n    \"moduleResolution\": \"bundler\",\n    \"allowImportingTsExtensions\": true,\n    \"resolveJsonModule\": true,\n    \"isolatedModules\": true,\n    \"noEmit\": true,\n    \"jsx\": \"react-jsx\",\n    \"strict\": true,\n    \"noUnusedLocals\": true,\n    \"noUnusedParameters\": true,\n    \"noFallthroughCasesInSwitch\": true\n  },\n  \"include\": [\"src\"],\n  \"references\": [{ \"path\": \"./tsconfig.node.json\" }]\n}\n".into() 
                },
                TemplateFile { 
                    path: "tsconfig.node.json".into(), 
                    content: "{\n  \"compilerOptions\": {\n    \"composite\": true,\n    \"skipLibCheck\": true,\n    \"module\": \"ESNext\",\n    \"moduleResolution\": \"bundler\",\n    \"allowSyntheticDefaultImports\": true\n  },\n  \"include\": [\"vite.config.ts\"]\n}\n".into() 
                },
                TemplateFile { 
                    path: "vite.config.ts".into(), 
                    content: "import { defineConfig } from 'vite';\nimport react from '@vitejs/plugin-react';\n\nexport default defineConfig({\n  plugins: [react()],\n});\n".into() 
                },
            ],
        },
        ProjectTemplate {
            id: "api-fastapi".into(),
            name: "FastAPI REST API".into(),
            description: "Python FastAPI backend with auto-generated docs".into(),
            language: "Python".into(),
            tags: vec!["api".into(), "backend".into(), "python".into()],
            structure: vec![
                TemplateFile { 
                    path: "main.py".into(), 
                    content: "from fastapi import FastAPI, HTTPException\nfrom pydantic import BaseModel\nfrom typing import List, Optional\nfrom datetime import datetime\n\napp = FastAPI(\n    title='{{project_name}}',\n    description='{{description}}',\n    version='0.1.0'\n)\n\n# Models\nclass Item(BaseModel):\n    id: Optional[int] = None\n    name: str\n    description: Optional[str] = None\n    created_at: datetime = datetime.now()\n\n# In-memory storage\nitems_db: List[Item] = []\n\n@app.get('/')\nasync def root():\n    \"\"\"Root endpoint\"\"\"\n    return {\n        'message': 'Hello from {{project_name}}!',\n        'docs': '/docs'\n    }\n\n@app.get('/items', response_model=List[Item])\nasync def get_items():\n    \"\"\"Get all items\"\"\"\n    return items_db\n\n@app.post('/items', response_model=Item)\nasync def create_item(item: Item):\n    \"\"\"Create a new item\"\"\"\n    item.id = len(items_db) + 1\n    items_db.append(item)\n    return item\n\n@app.get('/items/{item_id}', response_model=Item)\nasync def get_item(item_id: int):\n    \"\"\"Get a specific item by ID\"\"\"\n    for item in items_db:\n        if item.id == item_id:\n            return item\n    raise HTTPException(status_code=404, detail='Item not found')\n\nif __name__ == '__main__':\n    import uvicorn\n    uvicorn.run(app, host='0.0.0.0', port=8000)\n".into() 
                },
                TemplateFile { 
                    path: "requirements.txt".into(), 
                    content: "fastapi>=0.104.0\nuvicorn[standard]>=0.24.0\npydantic>=2.0.0\n".into() 
                },
                TemplateFile { 
                    path: "README.md".into(), 
                    content: "# {{project_name}}\n\n{{description}}\n\n## Setup\n\n```bash\npip install -r requirements.txt\n```\n\n## Run\n\n```bash\nuvicorn main:app --reload\n```\n\n## API Documentation\n\nOnce running, visit:\n- Swagger UI: http://localhost:8000/docs\n- ReDoc: http://localhost:8000/redoc\n\n## Test\n\n```bash\ncurl http://localhost:8000\ncurl http://localhost:8000/items\n```\n".into() 
                },
                TemplateFile { 
                    path: ".gitignore".into(), 
                    content: "__pycache__/\n*.py[cod]\n.env\n.venv\nvenv/\n*.log\n__pycache__/\n".into() 
                },
            ],
        },
        ProjectTemplate {
            id: "rust-lib".into(),
            name: "Rust Library".into(),
            description: "Rust crate with tests and documentation".into(),
            language: "Rust".into(),
            tags: vec!["rust".into(), "library".into(), "crate".into()],
            structure: vec![
                TemplateFile { 
                    path: "Cargo.toml".into(), 
                    content: "[package]\nname = \"{{project_name}}\"\nversion = \"0.1.0\"\nedition = \"2021\"\ndescription = \"{{description}}\"\nlicense = \"MIT\"\n\n[dependencies]\n\n[dev-dependencies]\n".into() 
                },
                TemplateFile { 
                    path: "src/lib.rs".into(), 
                    content: "//! {{description}}\n//!\n//! # Example\n//!\n//! ```\n//! let result = {{project_name}}::hello();\n//! assert_eq!(result, \"Hello from {{project_name}}!\");\n//! ```\n\n/// Example function that returns a greeting\npub fn hello() -> &'static str {\n    \"Hello from {{project_name}}!\"\n}\n\n/// Add two numbers together\npub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn test_hello() {\n        assert_eq!(hello(), \"Hello from {{project_name}}!\");\n    }\n\n    #[test]\n    fn test_add() {\n        assert_eq!(add(2, 2), 4);\n    }\n}\n".into() 
                },
                TemplateFile { 
                    path: "src/main.rs".into(), 
                    content: "//! {{project_name}} CLI\n\nfn main() {\n    println!(\"{}\", {{project_name}}::hello());\n}\n".into() 
                },
                TemplateFile { 
                    path: "README.md".into(), 
                    content: "# {{project_name}}\n\n{{description}}\n\n## Usage\n\nAdd to your `Cargo.toml`:\n\n```toml\n[dependencies]\n{{project_name}} = \"0.1.0\"\n```\n\nThen use in your code:\n\n```rust\nfn main() {\n    println!(\"{}\", {{project_name}}::hello());\n}\n```\n\n## Development\n\n```bash\ncargo test\ncargo doc --open\n```\n".into() 
                },
                TemplateFile { 
                    path: ".gitignore".into(), 
                    content: "/target\n**/*.rs.bk\nCargo.lock\n*.pdb\n".into() 
                },
            ],
        },
        ProjectTemplate {
            id: "nodejs-express".into(),
            name: "Node.js + Express API".into(),
            description: "Express.js REST API server with TypeScript".into(),
            language: "TypeScript/JavaScript".into(),
            tags: vec!["api".into(), "backend".into(), "node".into(), "express".into()],
            structure: vec![
                TemplateFile { 
                    path: "package.json".into(), 
                    content: "{\n  \"name\": \"{{project_name}}\",\n  \"version\": \"0.1.0\",\n  \"description\": \"{{description}}\",\n  \"main\": \"dist/index.js\",\n  \"scripts\": {\n    \"dev\": \"tsx watch src/index.ts\",\n    \"build\": \"tsc\",\n    \"start\": \"node dist/index.js\",\n    \"typecheck\": \"tsc --noEmit\"\n  },\n  \"dependencies\": {\n    \"express\": \"^4.18.2\"\n  },\n  \"devDependencies\": {\n    \"@types/express\": \"^4.17.21\",\n    \"@types/node\": \"^20.10.0\",\n    \"typescript\": \"~5.8.3\",\n    \"tsx\": \"^4.6.0\"\n  }\n}\n".into() 
                },
                TemplateFile { 
                    path: "tsconfig.json".into(), 
                    content: "{\n  \"compilerOptions\": {\n    \"target\": \"ES2022\",\n    \"module\": \"NodeNext\",\n    \"moduleResolution\": \"NodeNext\",\n    \"outDir\": \"./dist\",\n    \"rootDir\": \"./src\",\n    \"strict\": true,\n    \"esModuleInterop\": true,\n    \"skipLibCheck\": true,\n    \"forceConsistentCasingInFileNames\": true,\n    \"resolveJsonModule\": true\n  },\n  \"include\": [\"src/**/*\"],\n  \"exclude\": [\"node_modules\", \"dist\"]\n}\n".into() 
                },
                TemplateFile { 
                    path: "src/index.ts".into(), 
                    content: "import express, { Request, Response } from 'express';\n\nconst app = express();\nconst PORT = process.env.PORT || 3000;\n\napp.use(express.json());\n\n// Root endpoint\napp.get('/', (req: Request, res: Response) => {\n  res.json({\n    message: 'Hello from {{project_name}}!',\n    description: '{{description}}'\n  });\n});\n\n// Health check\napp.get('/health', (req: Request, res: Response) => {\n  res.json({ status: 'ok', timestamp: new Date().toISOString() });\n});\n\n// Start server\napp.listen(PORT, () => {\n  console.log(`🚀 {{project_name}} running on port ${PORT}`);\n  console.log(`📚 API docs: http://localhost:${PORT}`);\n});\n".into() 
                },
                TemplateFile { 
                    path: "README.md".into(), 
                    content: "# {{project_name}}\n\n{{description}}\n\n## Setup\n\n```bash\nnpm install\n```\n\n## Development\n\n```bash\nnpm run dev\n```\n\n## Build\n\n```bash\nnpm run build\nnpm start\n```\n\n## API Endpoints\n\n- `GET /` - Welcome message\n- `GET /health` - Health check\n".into() 
                },
                TemplateFile { 
                    path: ".gitignore".into(), 
                    content: "node_modules/\ndist/\n.env\n*.log\n".into() 
                },
            ],
        },
    ])
}

/// Returns loadable skills for Qwen AI
#[tauri::command]
pub async fn get_skills() -> Result<Vec<Skill>, String> {
    Ok(vec![
        Skill {
            id: "10x-engineer".into(),
            name: "🚀 10x Engineer Mode".into(),
            description: "Maximizes code delivery, architecture, and advanced patterns.".into(),
            prompt: "You are a 10x engineer. Deliver highly optimized, robust, and scalable code. Use best practices. Produce elegant architectural structures. Write thorough comments explaining complex logic, and make the code modular.".into(),
        },
        Skill {
            id: "ui-ux-designer".into(),
            name: "🎨 Premium UI/UX".into(),
            description: "Produces modern aesthetics, glassmorphism, responsive designs, and smooth animations.".into(),
            prompt: "You are an expert UI/UX developer. Prioritize beautiful, premium, and dynamic design aesthetics. Use modern web design principles like subtle glassmorphism, smooth animations/transitions, and responsive layouts. Ensure excellent accessibility.".into(),
        },
        Skill {
            id: "rust-expert".into(),
            name: "🦀 Senior Rustacean".into(),
            description: "Idiomatic Rust, error handling, safety, and max performance.".into(),
            prompt: "You are a senior Rust developer. Write highly idiomatic Rust code. Use modern language features. Ensure comprehensive error handling (e.g., anyhow, thiserror). Write thread-safe code. Add #![forbid(unsafe_code)] where applicable. Include detailed inline documentation and unit tests.".into(),
        },
        Skill {
            id: "ts-strict".into(),
            name: "📘 Strict TypeScript".into(),
            description: "Strict TS patterns, interfaces, and zero 'any' types.".into(),
            prompt: "Write strict TypeScript code. NEVER use 'any'. Explicitly type all function returns and arguments. Prefer interfaces. Use modern TS syntax including discriminated unions and mapped types.".into(),
        },
        Skill {
            id: "secure-coder".into(),
            name: "🔒 Security Auditor".into(),
            description: "Secure coding practices to prevent vulnerabilities.".into(),
            prompt: "You are a security expert. Prioritize secure coding practices. Prevent SQL injection, XSS, CSRF, and other common vulnerabilities. Validate all user inputs. Use secure defaults for cryptographic operations and secure communication.".into(),
        }
    ])
}

/// Scaffolds project files on disk, inits git, creates GitHub repo, pushes
#[tauri::command]
pub async fn build_and_push_project(
    req: BuildProjectRequest,
) -> Result<BuildResult, String> {
    let project_dir = format!("{}/{}", req.output_dir.trim_end_matches('/'), req.project_name);

    // 1. Create directory
    fs::create_dir_all(&project_dir)
        .map_err(|e| format!("Failed to create project dir: {e}"))?;

    // 2. Write all files
    for file in &req.generated_files {
        let full_path = format!("{}/{}", project_dir, file.path);
        // Create parent dirs
        if let Some(parent) = std::path::Path::new(&full_path).parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create dir {}: {e}", parent.display()))?;
        }
        fs::write(&full_path, &file.content)
            .map_err(|e| format!("Failed to write {}: {e}", file.path))?;
    }

    // 3. Write .gitignore
    let gitignore = "node_modules/\n.env\n*.log\ntarget/\n__pycache__/\n*.pyc\n.DS_Store\n";
    fs::write(format!("{project_dir}/.gitignore"), gitignore)
        .map_err(|e| format!("Failed to write .gitignore: {e}"))?;

    // 3.5 Run Pre-Push Checks
    // Removed based on user request

    // 4. Git init
    let git_init = Command::new("git")
        .args(["init"])
        .current_dir(&project_dir)
        .output()
        .map_err(|e| e.to_string())?;

    if !git_init.status.success() {
        return Err(String::from_utf8_lossy(&git_init.stderr).to_string());
    }

    // 5. Initial commit
    Command::new("git").args(["add", "-A"]).current_dir(&project_dir).output().map_err(|e| e.to_string())?;
    
    let safe_description = req.description.replace('\n', " ").replace('\r', "");
    
    let commit = Command::new("git")
        .args(["commit", "-m", &format!("Initial commit: {}", safe_description)])
        .current_dir(&project_dir)
        .output()
        .map_err(|e| e.to_string())?;

    if !commit.status.success() {
        return Err(String::from_utf8_lossy(&commit.stderr).to_string());
    }

    // 6. Create GitHub repo and push using gh CLI
    let visibility = if req.private_repo { "--private" } else { "--public" };
    let gh_create = Command::new("gh")
        .args([
            "repo", "create", &req.project_name,
            "--source", &project_dir,
            "--remote", "origin",
            "--push",
            visibility,
        ])
        .output()
        .map_err(|e| e.to_string())?;

    if !gh_create.status.success() {
        let err = String::from_utf8_lossy(&gh_create.stderr).to_string();
        // Still return success for local - just note the push failed
        return Ok(BuildResult {
            success: true,
            local_path: project_dir,
            repo_url: None,
            message: format!("Project created locally. GitHub push failed: {err}"),
        });
    }

    // 7. Get the repo URL
    let repo_url_output = Command::new("gh")
        .args(["repo", "view", "--json", "url", "--jq", ".url"])
        .current_dir(&project_dir)
        .output()
        .map_err(|e| e.to_string())?;

    let repo_url = String::from_utf8_lossy(&repo_url_output.stdout).trim().to_string();

    let repo_url_val = Some(repo_url);
    
    // 8. Track the project
    let _ = crate::commands::tracker::track_project_internal(crate::commands::tracker::ProjectRecord {
        id: uuid::Uuid::new_v4().to_string(),
        name: req.project_name.clone(),
        path: project_dir.clone(),
        remote_url: repo_url_val.clone(),
        tech_stack: vec![], // TODO: extract from template/request
        last_modified: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
    }).await;

    Ok(BuildResult {
        success: true,
        local_path: project_dir,
        repo_url: repo_url_val,
        message: format!("✅ {} created and pushed to GitHub!", req.project_name),
    })
}

/// Apply template placeholders to file content
pub fn apply_template(content: &str, project_name: &str, description: &str) -> String {
    content
        .replace("{{project_name}}", project_name)
        .replace("{{description}}", description)
}

/// Build GeneratedFiles from a template
#[tauri::command]
pub async fn template_to_files(
    template_id: String,
    project_name: String,
    description: String,
) -> Result<Vec<GeneratedFile>, String> {
    let templates = get_templates().await?;
    let template = templates.into_iter()
        .find(|t| t.id == template_id)
        .ok_or_else(|| format!("Template '{template_id}' not found"))?;

    let files = template.structure.into_iter().map(|f| GeneratedFile {
        path: f.path,
        content: apply_template(&f.content, &project_name, &description),
    }).collect();

    Ok(files)
}
