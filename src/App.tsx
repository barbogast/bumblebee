import { useState } from 'react';
import { Layout, Menu, Table, Alert } from 'antd';
import './App.css';
import { open } from '@tauri-apps/api/dialog';
import { invoke } from '@tauri-apps/api/tauri';
import { ColumnsType } from 'antd/es/table';

type EntryType = 'Directory' | 'File' | 'Link' | 'Unknown';
type Result = [
  {
    missing_in_dir_a: string[];
    missing_in_dir_b: string[];
  },
  {
    differing_content: string[];
    file_and_directory: { path: string; type_in_dir_a: EntryType; type_in_dir_b: EntryType }[];
  }
];

type Reason = 'missingInA' | 'missingInB' | 'differingContent' | 'fileAndDirectory';

type TableData = {
  key: string;
  path: string;
  reason: Reason;
};

const { Header, Content, Footer, Sider } = Layout;

const renderTableCell = (text: string, record: TableData) => {
  const classMap: { [key in Reason]: 'warning' | 'error' } = {
    missingInA: 'error',
    missingInB: 'error',
    differingContent: 'warning',
    fileAndDirectory: 'warning',
  };
  return text ? (
    <Alert type={classMap[record.reason]} message={text}>
      {record.reason}
    </Alert>
  ) : null;
};

const columns: ColumnsType<TableData> = [
  {
    title: 'Path',
    dataIndex: 'path',
    key: 'path',
  },
  {
    title: 'A',
    dataIndex: 'dirA',
    key: 'dirA',
    render: renderTableCell,
  },
  {
    title: 'B',
    dataIndex: 'dirB',
    key: 'dirB',
    render: renderTableCell,
  },
];

function App() {
  const [collapsed, setCollapsed] = useState(false);
  const [pathA, setPathA] = useState<string>('');
  const [pathB, setPathB] = useState<string>('');
  const [result, setResult] = useState<Result | void>();

  const missingInDirA: TableData[] = result
    ? result[0].missing_in_dir_a.map((path) => ({
        key: path,
        path,
        reason: 'missingInA',
        dirA: 'File missing',
      }))
    : [];

  const missingInDirB: TableData[] = result
    ? result[0].missing_in_dir_b.map((path) => ({
        key: path,
        path,
        reason: 'missingInB',
        dirB: 'File missing',
      }))
    : [];

  const differintContent: TableData[] = result
    ? result[1].differing_content.map((path) => ({
        key: path,
        path,
        reason: 'differingContent',
        dirA: 'Differing content',
        dirB: 'Differing content',
      }))
    : [];

  const fileAndDirectory: TableData[] = result
    ? result[1].file_and_directory.map((mismatch) => ({
        key: mismatch.path,
        path: mismatch.path,
        reason: 'fileAndDirectory',
        dirA: mismatch.type_in_dir_a,
        dirB: mismatch.type_in_dir_b,
      }))
    : [];

  const tableData = missingInDirA
    .concat(missingInDirB)
    .concat(differintContent)
    .concat(fileAndDirectory);

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

                  invoke('compare', { pathA, pathB })
                    .then((message) => setResult(message as Result))
                    .catch((e) => console.error(e));
                }}
              >
                Compare
              </button>
            </div>
            {result && <Table dataSource={tableData} columns={columns} />}
          </div>
        </Content>
        <Footer style={{ textAlign: 'center' }}>Ant Design ©2018 Created by Ant UED</Footer>
      </Layout>
    </Layout>
  );
}

export default App;
