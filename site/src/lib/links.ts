export const EXE =
  'https://github.com/pjmagee/wwm/releases/latest/download/wwm.exe';
export const RELEASES = 'https://github.com/pjmagee/wwm/releases';
export const REPO = 'https://github.com/pjmagee/wwm';

export const INSTALL_PS = [
  'New-Item $HOME\\.wwm -ItemType Directory -Force | Out-Null',
  `Invoke-WebRequest -UseBasicParsing -Uri ${EXE} -OutFile $HOME\\.wwm\\wwm.exe`,
  '$env:Path = "$HOME\\.wwm;$env:Path"',
  'wwm',
].join('\n');
