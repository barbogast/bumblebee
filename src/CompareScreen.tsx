import { useState } from 'react';
import { Button } from 'antd';
import { DoubleRightOutlined, DoubleLeftOutlined } from '@ant-design/icons';
import './App.css';
import { open } from '@tauri-apps/api/dialog';
import { invoke } from '@tauri-apps/api/tauri';

import CopyModal, { useModalState } from './CopyModal';
import ComparisonTable, { useTableState } from './ComparisonTable';
import { CompareResult } from './types';

const CompareScreen = () => {
  const [pathA, setPathA] = useState<string>('');
  const [pathB, setPathB] = useState<string>('');
  const tableApi = useTableState();
  const modalApi = useModalState();

  return (
    <>
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
