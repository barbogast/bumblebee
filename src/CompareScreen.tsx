import { useState } from 'react';
import { Button } from 'antd';
import { DoubleRightOutlined, DoubleLeftOutlined } from '@ant-design/icons';
import './App.css';
import { invoke } from '@tauri-apps/api/tauri';

import CopyModal, { useModalState } from './CopyModal';
import ComparisonTable, { useTableState } from './ComparisonTable';
import { CompareResult } from './types';
import DirectorySelect from './DirectorySelect';

const CompareScreen = () => {
  const [pathA, setPathA] = useState<string>('');
  const [pathB, setPathB] = useState<string>('');
  const tableApi = useTableState();
  const modalApi = useModalState();

  return (
    <>
      <DirectorySelect value={pathA} onChange={setPathA} buttonLabel='Set directory A' />
      <DirectorySelect value={pathB} onChange={setPathB} buttonLabel='Set directory B' />

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
            Copy A to B...
          </Button>
          <Button
            type='primary'
            size='large'
            icon={<DoubleLeftOutlined />}
            onClick={() => modalApi.openModal(pathB, pathA, tableApi.selectedRows)}
          >
            Copy B to A...
          </Button>
        </div>
      ) : null}

      <CopyModal modalApi={modalApi} />
    </>
  );
};

export default CompareScreen;
