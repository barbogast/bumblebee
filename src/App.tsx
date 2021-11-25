import { useState } from 'react';
import './App.css';
import { open } from '@tauri-apps/api/dialog';
import { invoke } from '@tauri-apps/api/tauri';

function App() {
  const [pathA, setPathA] = useState<string>();
  const [pathB, setPathB] = useState<string>();
  return (
    <div className='App'>
      <header className='App-header'>
        {pathA}
        <button
          onClick={() =>
            open({ directory: true })
              .then((path) => setPathA(path as string))
              .catch(console.error)
          }
        >
          Set dir A
        </button>
        {pathB}
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
              .then((message) => console.log(message))
              .catch((e) => console.error(e));
          }}
        >
          Compare
        </button>
      </header>
    </div>
  );
}

export default App;
