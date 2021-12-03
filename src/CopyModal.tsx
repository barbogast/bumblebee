import { useState } from 'react';
import { Alert, Modal } from 'antd';
import { invoke } from '@tauri-apps/api/tauri';

type ErrorInfo = {
  message: string;
  path: string;
};

type ModalState = {
  sourcePath: string;
  targetPath: string;
  selectedEntries: string[];
  copyErrors: ErrorInfo[];
};

type ModalApi = ReturnType<typeof useModalState>;

const Path = ({ children }: { children: React.ReactNode }) => (
  <code
    style={{ fontSize: 12, background: 'lightgrey', padding: 5, borderRadius: 2, display: 'block' }}
  >
    {children}
  </code>
);

export const useModalState = () => {
  const [state, setState] = useState<ModalState | void>();

  const openModal = (sourcePath: string, targetPath: string, selectedEntries: string[]) =>
    setState({
      sourcePath,
      targetPath,
      selectedEntries,
      copyErrors: [],
    });

  const closeModal = () => setState();

  const setCopyErrors = (copyErrors: ErrorInfo[]) => {
    if (!state) return;
    setState({ ...state, copyErrors });
  };

  return { openModal, closeModal, state, setCopyErrors };
};

type Props = {
  modalApi: ModalApi;
};

const CopyModal = ({ modalApi }: Props) => {
  if (!modalApi.state) {
    return null;
  }
  const { sourcePath, targetPath, selectedEntries, copyErrors } = modalApi.state;

  const onOk = () =>
    invoke<ErrorInfo[]>('copy', { sourcePath, targetPath, subPaths: selectedEntries })
      .then((errors) => {
        if (errors.length) {
          modalApi.setCopyErrors(errors);
        } else {
          modalApi.closeModal();
        }
      })
      .catch((e) => modalApi.setCopyErrors([{ path: '', message: e }]));

  return (
    <Modal title='Title' visible onOk={onOk} onCancel={() => modalApi.closeModal()}>
      Source directory
      <br /> <Path>{sourcePath}</Path>
      <br /> Target directory
      <br /> <Path>{targetPath}</Path>
      <br />
      Files
      {selectedEntries.map((e) => (
        <Path>{e}</Path>
      ))}
      {copyErrors.length ? (
        <Alert
          type='error'
          message={
            <>
              While copying / overriding the following errors occured:
              {copyErrors.map((e) => (
                <p>
                  {e.path ? e.path + ' : ' : null} {e.message}
                </p>
              ))}
            </>
          }
        />
      ) : null}
    </Modal>
  );
};

export default CopyModal;
