import { useState } from 'react';
import { Table } from 'antd';
import { invoke } from '@tauri-apps/api/tauri';
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
  key: string;
  children: AntTreeNode[] | void;
};

const convertNode = (node: Node): AntTreeNode => {
  return {
    name: node.path.split('/').pop() || '',
    size: node.size,
    sizeHuman: filesize(node.size),
    numberOfFiles: node.type === 'Dir' ? String(node.number_of_files) : '',
    key: node.path,
    children: node.type === 'Dir' ? node.content.map(convertNode) : undefined,
  };
};

const DiskSpaceScreen = () => {
  const [path, setPath] = useState('');
  const [result, setResult] = useState<AntTreeNode[] | void>();

  return (
    <>
      <DirectorySelect value={path} onChange={setPath} buttonLabel='Select directory' />
      <button
        onClick={() =>
          invoke<DirectoryNode>('analyze_disk_usage', { path })
            .then((res) => res.content.map(convertNode))
            .then(setResult)
            .catch(console.error)
        }
      >
        Analyze!
      </button>
      {result ? (
        <Table
          columns={[
            { title: 'Name', dataIndex: 'name' },
            { title: 'Size', dataIndex: 'sizeHuman', sorter: (a, b) => a.size - b.size },
            {
              title: '# files',
              dataIndex: 'numberOfFiles',
              sorter: (a, b) => parseInt(a.numberOfFiles) - parseInt(b.numberOfFiles),
            },
          ]}
          dataSource={result}
        />
      ) : null}
    </>
  );
};

export default DiskSpaceScreen;
