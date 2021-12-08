import { useEffect, useState } from 'react';
import { Table } from 'antd';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import filesize from 'filesize';
import DirectorySelect from './DirectorySelect';

type FileNode = {
  type: 'File';
  path: string;
  size: number;
};

type DirectoryNode = {
  type: 'Dir';
  path: string;
  size: number;
  number_of_files: number;
  content: Node[];
};

type Node = FileNode | DirectoryNode;

type AntTreeNode = {
  name: string;
  size: number;
  sizeHuman: string;
  numberOfFiles: string;
  numberOfFilesHuman: string;
  key: string;
  children: AntTreeNode[] | void;
};

const convertNode = (node: Node): AntTreeNode => {
  return {
    name: node.path.split('/').pop() || '',
    size: node.size,
    sizeHuman: filesize(node.size),
    numberOfFiles: node.type === 'Dir' ? String(node.number_of_files) : '',
    numberOfFilesHuman: node.type === 'Dir' ? node.number_of_files.toLocaleString() : '',
    key: node.path,
    children: node.type === 'Dir' ? node.content.map(convertNode) : undefined,
  };
};

const replaceNode = (parentNode: AntTreeNode, newNode: Node): AntTreeNode => {
  if (parentNode.key === newNode.path) {
    return convertNode(newNode);
  } else {
    return {
      ...parentNode,
      children: parentNode.children?.map((child) => replaceNode(child, newNode)),
    };
  }
};

const DiskSpaceScreen = () => {
  const [path, setPath] = useState('');
  const [result, setResult] = useState<AntTreeNode[] | void>();
  const [durationBE, setDurationBE] = useState<number | void>();
  const [durationFE, setDurationFE] = useState<number | void>();
  const [progress, setProgress] = useState<{
    path: string;
    numberOfFiles: number;
    totalSize: number;
  } | void>();

  useEffect(() => {
    const unlisten = listen<{
      path: string;
      number_of_files_found: number;
      total_size_found: number;
    }>('progress', (event) => {
      const {
        path: currentPath,
        number_of_files_found: numberOfFiles,
        total_size_found: totalSize,
      } = event.payload;
      setProgress({ path: currentPath.slice(path.length + 1), numberOfFiles, totalSize });
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, [path.length]);

  return (
    <>
      <DirectorySelect value={path} onChange={setPath} buttonLabel='Select directory' />
      <button
        onClick={() => {
          setDurationBE(undefined);
          setDurationFE(undefined);
          setResult(undefined);
          let start = Date.now();
          invoke<{ result: DirectoryNode; duration: number }>('analyze_disk_usage', { path })
            .then((res) => {
              setDurationBE(res.duration);
              setDurationFE(Date.now() - start);
              setProgress({
                path: '',
                numberOfFiles: res.result.number_of_files,
                totalSize: res.result.size,
              });
              return res.result.content.map(convertNode);
            })
            .then(setResult)
            .catch(console.error);
        }}
      >
        Analyze!
      </button>
      <button onClick={() => invoke('abort').catch(console.error)}>Abort</button>
      {progress ? (
        <div>
          <div>Files: {progress.numberOfFiles.toLocaleString()}</div>
          <div>Size: {filesize(progress.totalSize)}</div>
          <div style={{ textOverflow: 'ellipsis', overflow: 'hidden', whiteSpace: 'nowrap' }}>
            {progress.path}
          </div>
        </div>
      ) : null}
      {durationBE ? <div>Duration BE: {durationBE}</div> : null}
      {durationFE ? <div>Duration FE: {durationFE}</div> : null}
      {result ? (
        <Table
          columns={[
            { title: 'Name', dataIndex: 'name' },
            { title: 'Size', dataIndex: 'sizeHuman', sorter: (a, b) => a.size - b.size },
            {
              title: '# files',
              dataIndex: 'numberOfFilesHuman',
              sorter: (a, b) => parseInt(a.numberOfFiles) - parseInt(b.numberOfFiles),
            },
          ]}
          dataSource={result}
          expandable={{
            onExpand: (expanded, record) => {
              if (expanded) {
                invoke<DirectoryNode | void>('load_nested_directory', { path: record.key })
                  .then((newNode) => {
                    if (!newNode) {
                      return;
                    }
                    setResult((previousRes) => previousRes?.map((p) => replaceNode(p, newNode)));
                  })
                  .catch(console.error);
              }
            },
          }}
        />
      ) : null}
    </>
  );
};

export default DiskSpaceScreen;
