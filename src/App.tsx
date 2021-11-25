import { useState } from 'react';
import './App.css';
import { open } from '@tauri-apps/api/dialog';
import { invoke } from '@tauri-apps/api/tauri';

type Result = [
  {
    missing_in_dir_a: string[];
    missing_in_dir_b: string[];
  },
  { differing_content: string[]; file_and_directory: string[] }
];

function App() {
  const [pathA, setPathA] = useState<string>();
  const [pathB, setPathB] = useState<string>();
  const [result, setResult] = useState<Result | void>();
  return (
    <div className='App'>
      <header className='App-header'>
        <input value={pathB} onChange={(e) => setPathA(e.target.value)} />
        <button
          onClick={() =>
            open({ directory: true })
              .then((path) => setPathA(path as string))
              .catch(console.error)
          }
        >
          Set dir A
        </button>
        <input value={pathB} onChange={(e) => setPathB(e.target.value)} />
        <button
          onClick={() =>
            open({ directory: true })
              .then((path) => setPathB(path as string))
              .catch(console.error)
          }
        >
          Set dir B
        </button>
        <button
          onClick={() => {
            console.log('invoke');

            invoke('compare', { pathA, pathB })
              .then((message) => setResult(message as Result))
              .catch((e) => console.error(e));
          }}
        >
          Compare
        </button>
        missing_in_dir_a
        {result && <textarea value={result[0].missing_in_dir_a.join('\n')} />}
        missing_in_dir_b
        {result && <textarea value={result[0].missing_in_dir_b.join('\n')} />}
        differing_content
        {result && <textarea value={result[1].differing_content.join('\n')} />}
        file_and_directory
        {result && <textarea value={result[1].file_and_directory.join('\n')} />}
      </header>
    </div>
  );
}

export default App;
