import { useState } from 'react';
import { Layout, Menu, Button } from 'antd';
import { DoubleRightOutlined, DoubleLeftOutlined } from '@ant-design/icons';
import './App.css';
import { open } from '@tauri-apps/api/dialog';
import { invoke } from '@tauri-apps/api/tauri';

import CopyModal, { useModalState } from './CopyModal';
import ComparisonTable, { useTableState } from './ComparisonTable';
import { CompareResult } from './types';

const { Header, Content, Footer, Sider } = Layout;

function App() {
  const [collapsed, setCollapsed] = useState(false);
  const [pathA, setPathA] = useState<string>('');
  const [pathB, setPathB] = useState<string>('');
  const tableApi = useTableState();
  const modalApi = useModalState();
  console.log(modalApi.state);
  console.log(tableApi);

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
            <div style={{ display: 'flex', flexDirection: 'row', marginBottom: 10 }}>
              <button
                onClick={() =>
                  open({ directory: true })
                    .then((path) => setPathA(path as string))
                    .catch(console.error)
                }
                style={{ marginRight: 10 }}
              >
                Set directory A
              </button>
              <input value={pathA} onChange={(e) => setPathA(e.target.value)} style={{ flex: 1 }} />
            </div>
            <div style={{ display: 'flex', flexDirection: 'row', marginBottom: 10 }}>
              <button
                onClick={() =>
                  open({ directory: true })
                    .then((path) => setPathB(path as string))
                    .catch(console.error)
                }
                style={{ marginRight: 10 }}
              >
                Set directory B
              </button>
              <input value={pathB} onChange={(e) => setPathB(e.target.value)} style={{ flex: 1 }} />
            </div>
            <div style={{ marginBottom: 10 }}>
              <button
                onClick={() => {
                  console.log('invoke');

                  invoke<CompareResult>('compare', { pathA, pathB })
                    .then((message) => {
                      tableApi.setComparisonResult(message);
                    })
                    .catch((e) => console.error(e));
                }}
              >
                Compare
              </button>
            </div>

            <ComparisonTable tableApi={tableApi} />

            {tableApi.selectedRows.length ? (
              <div style={{ padding: 10 }}>
                <Button
                  type='primary'
                  size='large'
                  icon={<DoubleRightOutlined />}
                  style={{ marginRight: 10 }}
                  onClick={() => modalApi.openModal(pathA, pathB, tableApi.selectedRows)}
                >
                  Copy selected files from directory A to directory B...
                </Button>
                <Button
                  type='primary'
                  size='large'
                  icon={<DoubleLeftOutlined />}
                  onClick={() => modalApi.openModal(pathB, pathA, tableApi.selectedRows)}
                >
                  Copy selected files from directory B to directory A...
                </Button>
              </div>
            ) : null}

            <CopyModal modalApi={modalApi} />
          </div>
        </Content>
        <Footer style={{ textAlign: 'center' }}>XAnt Design ©2018 Created by Ant UED</Footer>
      </Layout>
    </Layout>
  );
}

export default App;
