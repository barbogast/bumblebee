import { useState } from 'react';
import { Table, Alert } from 'antd';
import { ColumnsType } from 'antd/es/table';

import { Reason, CompareResult } from './types';

type TableData = {
  key: string;
  path: string;
  type: Reason;
};

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
      return 'warning';
    default: {
      const exhaustiveCheck: never = type;
      throw new Error(`Unhandled case: ${exhaustiveCheck}`);
    }
  }
};

export const useTableState = () => {
  const [comparisonResult, setComparisonResult] = useState<CompareResult>([]);
  const [selectedRowKeys, setSelectedRowKeys] = useState<string[]>([]);

  const setComparisonResultPublic = (comparisonResult: CompareResult) => {
    setComparisonResult(comparisonResult);
    setSelectedRowKeys(
      comparisonResult.filter((r) => (isAutoFixable(r.type) ? r.path : '')).map((r) => r.path)
    );
  };

  return {
    comparisonResult,
    setComparisonResult: setComparisonResultPublic,
    selectedRows: selectedRowKeys,
    setSelectedRowKeys, // TODO: Check for isAUtoFixable
  };
};

export type TableApi = ReturnType<typeof useTableState>;

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

type Props = {
  tableApi: TableApi;
};

const ComparisonTable = ({ tableApi }: Props) => {
  const { comparisonResult, selectedRows: selectedRowKeys } = tableApi;
  const tableData: TableData[] = comparisonResult
    ? comparisonResult.map((res) => {
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
              dirA: 'Missing',
            };
          }
          case 'MissingInDirB': {
            return {
              key: res.path,
              path: res.path,
              type: res.type,
              dirB: 'Missing',
            };
          }
          case 'DifferingContent': {
            return {
              key: res.path,
              path: res.path,
              type: res.type,
              dirA: `Differing content (${
                res.last_modified_in_dir_a > res.last_modified_in_dir_b ? 'newer' : 'older'
              })`,
              dirB: `Differing content (${
                res.last_modified_in_dir_a < res.last_modified_in_dir_b ? 'newer' : 'older'
              })`,
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

  return (
    <Table
      dataSource={tableData}
      columns={columns}
      rowSelection={{
        selectedRowKeys,
        // @ts-expect-error: Types say that number[] will be passed, even though it actually is string.
        onChange: (selection) => tableApi.setSelectedRowKeys(selection),
        ...rowSelection,
      }}
      pagination={{ size: 'small', hideOnSinglePage: true }}
    />
  );
};

export default ComparisonTable;
