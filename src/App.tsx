import { useState } from 'react';
import { Layout, Menu, Table, Alert, Button, Modal } from 'antd';
import { DoubleRightOutlined, DoubleLeftOutlined } from '@ant-design/icons';
import './App.css';
import { open } from '@tauri-apps/api/dialog';
import { invoke } from '@tauri-apps/api/tauri';
import { ColumnsType } from 'antd/es/table';

type EntryType = 'Directory' | 'File' | 'Link' | 'Unknown';

type ErrorInfo = {
  message: string;
  path: string;
};

type CompareResult = (
  | {
      type: 'CouldNotReadDirectory';
      path: string;
      message: string;
    }
  | {
      type: 'CouldNotCalculateHash';
      path: string;
      message: string;
    }
  | {
      type: 'MissingInDirA';
      path: string;
    }
  | {
      type: 'MissingInDirB';
      path: string;
    }
  | {
      type: 'DifferingContent';
      path: string;
    }
  | {
      type: 'TypeMismatch';
      path: string;
      type_in_dir_a: EntryType;
      type_in_dir_b: EntryType;
    }
)[];

type Reason =
  | 'CouldNotReadDirectory'
  | 'CouldNotCalculateHash'
  | 'MissingInDirA'
  | 'MissingInDirB'
  | 'DifferingContent'
  | 'TypeMismatch';

type TableData = {
  key: string;
  path: string;
  type: Reason;
};

const { Header, Content, Footer, Sider } = Layout;

const reasonToType = (record: TableData) => {
  const { type } = record;
  switch (type) {
    case 'CouldNotCalculateHash':
      return 'error';
    case 'CouldNotReadDirectory':
      return 'error';
    case 'MissingInDirA':
      return 'error';
    case 'MissingInDirB':
      return 'error';
    case 'DifferingContent':
      return 'warning';
    case 'TypeMismatch':
      return 'error';
    default: {
      const exhaustiveCheck: never = type;
      throw new Error(`Unhandled case: ${exhaustiveCheck}`);
    }
  }
};

const renderTableCell = (text: string, record: TableData) => {
  return text ? (
    <Alert type={reasonToType(record)} message={text}>
      {record.type}
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

const isAutoFixable = (type: Reason) => type !== 'TypeMismatch';

const rowSelection = {
  getCheckboxProps: (record: TableData) => ({
    disabled: !isAutoFixable(record.type),
    name: record.path,
    title: isAutoFixable(record.type) ? '' : `Can't be fixed automatically.`,
  }),
};

type ModalState = {
  sourcePath: string;
  targetPath: string;
  selectedEntries: string[];
  copyErrors: ErrorInfo[];
};

const useModalState = () => {
  const [state, setState] = useState<ModalState | void>();

  const openModal = (sourcePath: string, targetPath: string, selectedEntries: string[]) =>
    setState({
      sourcePath,
      targetPath,
      selectedEntries,
      copyErrors: [],
    });

  const closeModal = () => {
    setState();
  };

  const setCopyErrors = (copyErrors: ErrorInfo[]) => {
    if (!state) return;
    setState({ ...state, copyErrors });
  };

  return { openModal, closeModal, modalState: state, setCopyErrors };
};

function App() {
  const [collapsed, setCollapsed] = useState(false);
  const [pathA, setPathA] = useState<string>('');
  const [pathB, setPathB] = useState<string>('');
  const [result, setResult] = useState<CompareResult | void>();
  const [selectedRowKeys, setSelectedRowKeys] = useState<string[]>([]);
  const { openModal, closeModal, modalState, setCopyErrors } = useModalState();
  console.log(modalState);
  console.log(result);

  const errors: TableData[] = result
    ? result.map((res) => {
        const { type } = res;
        switch (type) {
          case 'CouldNotReadDirectory':
          case 'CouldNotCalculateHash': {
            return {
              key: res.path,
              path: res.path,
              type: res.type,
              dirA: res.message,
            };
          }
          case 'MissingInDirA': {
            return {
              key: res.path,
              path: res.path,
              type: res.type,
              dirA: 'File  missing',
            };
          }
          case 'MissingInDirB': {
            return {
              key: res.path,
              path: res.path,
              type: res.type,
              dirA: 'File missing',
            };
          }
          case 'DifferingContent': {
            return {
              key: res.path,
              path: res.path,
              type: res.type,
              dirA: 'Differing content',
              dirB: 'Differing content',
            };
          }
          case 'TypeMismatch': {
            return {
              key: res.path,
              path: res.path,
              type: res.type,
              dirA: res.type_in_dir_a,
              dirB: res.type_in_dir_b,
            };
          }
          default: {
            const exhaustiveCheck: never = type;
            throw new Error(`Unhandled case: ${exhaustiveCheck}`);
          }
        }
      })
    : [];

  const tableData = errors;

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
                      setResult(message);
                      setSelectedRowKeys(message.map((r) => (isAutoFixable(r.type) ? r.path : '')));
                    })
                    .catch((e) => console.error(e));
                }}
              >
                Compare
              </button>
            </div>
            {result && (
              <Table
                dataSource={tableData}
                columns={columns}
                rowSelection={{
                  selectedRowKeys,
                  // @ts-expect-error: Types say that number[] will be passed, even though it actually is string.
                  onChange: (selection) => setSelectedRowKeys(selection),
                  ...rowSelection,
                }}
                pagination={{ size: 'small', hideOnSinglePage: true }}
              />
            )}
            {selectedRowKeys.length ? (
              <div style={{ padding: 10 }}>
                <Button
                  type='primary'
                  size='large'
                  icon={<DoubleRightOutlined />}
                  style={{ marginRight: 10 }}
                  onClick={() => openModal(pathA, pathB, selectedRowKeys)}
                >
                  Copy selected files from directory A to directory B...
                </Button>
                <Button
                  type='primary'
                  size='large'
                  icon={<DoubleLeftOutlined />}
                  onClick={() => openModal(pathB, pathA, selectedRowKeys)}
                >
                  Copy selected files from directory B to directory A...
                </Button>
              </div>
            ) : null}
            {modalState ? (
              <Modal
                title='Title'
                visible
                onOk={() =>
                  invoke<ErrorInfo[]>('copy', {
                    pathSource: modalState.sourcePath,
                    pathTarget: modalState.targetPath,
                    subPaths: selectedRowKeys,
                  })
                    .then((errors) => {
                      if (!errors.length) {
                        closeModal();
                      } else {
                        setCopyErrors(errors);
                      }
                    })
                    .catch((e) => setCopyErrors([{ path: '', message: e }]))
                }
                onCancel={() => {
                  closeModal();
                }}
              >
                Copy/override the following paths from {modalState.sourcePath} to{' '}
                {modalState.targetPath}
                {selectedRowKeys.map((e) => (
                  <p>{e}</p>
                ))}
                {modalState.copyErrors.length ? (
                  <Alert
                    type='error'
                    message={
                      <>
                        While copying / overriding the following errors occured:
                        {modalState.copyErrors.map((e) => (
                          <p>
                            {e.path ? e.path + ' : ' : null} {e.message}
                          </p>
                        ))}
                      </>
                    }
                  />
                ) : null}
              </Modal>
            ) : null}
          </div>
        </Content>
        <Footer style={{ textAlign: 'center' }}>XAnt Design ©2018 Created by Ant UED</Footer>
      </Layout>
    </Layout>
  );
}

export default App;
