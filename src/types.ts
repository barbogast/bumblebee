export type EntryType = 'Directory' | 'File' | 'Link' | 'Unknown';

export type CompareResult = (
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

export type Reason =
  | 'CouldNotReadDirectory'
  | 'CouldNotCalculateHash'
  | 'MissingInDirA'
  | 'MissingInDirB'
  | 'DifferingContent'
  | 'TypeMismatch';
