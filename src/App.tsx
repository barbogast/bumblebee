import { useState } from 'react';
import { Layout, Menu } from 'antd';
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

const { Header, Content, Footer, Sider } = Layout;

function App() {
  const [collapsed, setCollapsed] = useState(false);
  const [pathA, setPathA] = useState<string>();
  const [pathB, setPathB] = useState<string>();
  const [result, setResult] = useState<Result | void>();
  return (
    <Layout style={{ minHeight: '100vh' }}>
      <Sider collapsible collapsed={collapsed} onCollapse={() => setCollapsed(!collapsed)}>
        <div className='logo' />
        <Menu theme='dark' defaultSelectedKeys={['1']} mode='inline'>
          <Menu.Item key='1'>Compare Directories</Menu.Item>
          <Menu.Item key='2'>Find duplicate files</Menu.Item>
        </Menu>
      </Sider>
      <Layout className='site-layout'>
        <Header className='site-layout-background' style={{ padding: 0 }} />
        <Content style={{ margin: '0 16px' }}>
          <div className='site-layout-background' style={{ padding: 24, minHeight: 360 }}>
            <div>
              <button
                onClick={() =>
                  open({ directory: true })
                    .then((path) => setPathA(path as string))
                    .catch(console.error)
                }
              >
                Set dir A
              </button>
              <input value={pathA} onChange={(e) => setPathA(e.target.value)} />
            </div>
            <div>
              <button
                onClick={() =>
                  open({ directory: true })
                    .then((path) => setPathB(path as string))
                    .catch(console.error)
                }
              >
                Set dir B
              </button>
              <input value={pathB} onChange={(e) => setPathB(e.target.value)} />
            </div>
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
            <div>
              missing_in_dir_a
              {result && <textarea value={result[0].missing_in_dir_a.join('\n')} disabled />}
            </div>
            <div>
              missing_in_dir_b
              {result && <textarea value={result[0].missing_in_dir_b.join('\n')} disabled />}
            </div>
            <div>
              differing_content
              {result && <textarea value={result[1].differing_content.join('\n')} disabled />}
            </div>
            <div>
              file_and_directory
              {result && <textarea value={result[1].file_and_directory.join('\n')} disabled />}
            </div>
          </div>
        </Content>
        <Footer style={{ textAlign: 'center' }}>Ant Design ©2018 Created by Ant UED</Footer>
      </Layout>
    </Layout>
  );
}

export default App;
