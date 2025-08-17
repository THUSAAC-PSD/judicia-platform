import React, { useEffect, useRef, useState } from 'react';
import { Upload, Download, Copy, Check, File } from 'lucide-react';
import { useDropzone } from 'react-dropzone';

interface CodeEditorProps {
  value: string;
  onChange: (value: string) => void;
  language?: string;
  placeholder?: string;
  className?: string;
  onFileUpload?: (content: string, filename: string) => void;
}

const LANGUAGE_MAP: Record<string, string> = {
  'javascript': 'js',
  'typescript': 'js',
  'python': 'py',
  'java': 'java',
  'cpp': 'cpp',
  'c++': 'cpp',
  'c': 'c',
  'csharp': 'cs',
  'c#': 'cs',
  'go': 'go',
  'rust': 'rs'
};

const LANGUAGE_EXTENSIONS: Record<string, string> = {
  'javascript': '.js',
  'typescript': '.ts',
  'python': '.py',
  'java': '.java',
  'cpp': '.cpp',
  'c++': '.cpp',
  'c': '.c',
  'csharp': '.cs',
  'c#': '.cs',
  'go': '.go',
  'rust': '.rs'
};

export default function CodeEditor({ 
  value, 
  onChange, 
  language = 'cpp', 
  placeholder = 'Enter your code here...',
  className = '',
  onFileUpload
}: CodeEditorProps) {
  const editorRef = useRef<HTMLTextAreaElement>(null);
  const preRef = useRef<HTMLPreElement>(null);
  const [copied, setCopied] = useState(false);
  const [uploadedFile, setUploadedFile] = useState<string | null>(null);

  const { getRootProps, getInputProps, isDragActive } = useDropzone({
    accept: {
      'text/plain': ['.txt', '.py', '.js', '.java', '.cpp', '.c', '.cs', '.go', '.rs', '.ts'],
      'application/x-python-code': ['.py'],
      'application/javascript': ['.js'],
      'text/x-java-source': ['.java'],
      'text/x-c++src': ['.cpp'],
      'text/x-csrc': ['.c'],
      'text/x-csharp': ['.cs'],
      'text/x-go': ['.go'],
      'text/x-rust': ['.rs']
    },
    multiple: false,
    onDrop: (acceptedFiles) => {
      if (acceptedFiles.length > 0) {
        const file = acceptedFiles[0];
        const reader = new FileReader();
        reader.onload = (e) => {
          const content = e.target?.result as string;
          onChange(content);
          setUploadedFile(file.name);
          if (onFileUpload) {
            onFileUpload(content, file.name);
          }
        };
        reader.readAsText(file);
      }
    }
  });

  useEffect(() => {
    // Simple syntax highlighting without external dependencies
    // Using basic CSS classes for basic highlighting
  }, [value, language]);

  const handleScroll = (e: React.UIEvent<HTMLTextAreaElement>) => {
    if (preRef.current) {
      preRef.current.scrollTop = e.currentTarget.scrollTop;
      preRef.current.scrollLeft = e.currentTarget.scrollLeft;
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Tab') {
      e.preventDefault();
      const textarea = e.currentTarget;
      const start = textarea.selectionStart;
      const end = textarea.selectionEnd;
      const newValue = value.substring(0, start) + '  ' + value.substring(end);
      onChange(newValue);
      
      setTimeout(() => {
        textarea.selectionStart = textarea.selectionEnd = start + 2;
      }, 0);
    }
  };

  const copyToClipboard = async () => {
    try {
      await navigator.clipboard.writeText(value);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('Failed to copy code:', err);
    }
  };

  const downloadCode = () => {
    const extension = LANGUAGE_EXTENSIONS[language] || '.txt';
    const filename = uploadedFile ? uploadedFile : `solution${extension}`;
    const blob = new Blob([value], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  const prismLanguage = LANGUAGE_MAP[language] || language;

  return (
    <div className={`relative border border-gray-300 rounded-lg overflow-hidden bg-gray-900 ${className}`}>
      {/* Toolbar */}
      <div className="bg-gray-800 px-4 py-2 flex items-center justify-between">
        <div className="flex items-center space-x-2">
          <span className="text-sm font-medium text-gray-300">Language: {language.toUpperCase()}</span>
          {uploadedFile && (
            <div className="flex items-center text-sm text-green-400">
              <File className="w-4 h-4 mr-1" />
              {uploadedFile}
            </div>
          )}
        </div>
        <div className="flex items-center space-x-2">
          <div {...getRootProps()} className="cursor-pointer">
            <input {...getInputProps()} />
            <button
              className="p-2 text-gray-400 hover:text-white hover:bg-gray-700 rounded transition-colors"
              title="Upload file"
            >
              <Upload className="w-4 h-4" />
            </button>
          </div>
          <button
            onClick={downloadCode}
            className="p-2 text-gray-400 hover:text-white hover:bg-gray-700 rounded transition-colors"
            title="Download code"
          >
            <Download className="w-4 h-4" />
          </button>
          <button
            onClick={copyToClipboard}
            className="p-2 text-gray-400 hover:text-white hover:bg-gray-700 rounded transition-colors"
            title="Copy code"
          >
            {copied ? <Check className="w-4 h-4 text-green-400" /> : <Copy className="w-4 h-4" />}
          </button>
        </div>
      </div>

      {/* Editor Container */}
      <div className="relative">
        {/* Drag and Drop Overlay */}
        {isDragActive && (
          <div className="absolute inset-0 bg-blue-500 bg-opacity-20 border-2 border-dashed border-blue-400 flex items-center justify-center z-10">
            <div className="text-blue-600 text-center">
              <Upload className="w-8 h-8 mx-auto mb-2" />
              <p className="text-lg font-medium">Drop your code file here</p>
            </div>
          </div>
        )}

        {/* Code Editor */}
        <div className="relative">
          <textarea
            ref={editorRef}
            value={value}
            onChange={(e) => onChange(e.target.value)}
            onScroll={handleScroll}
            onKeyDown={handleKeyDown}
            placeholder={placeholder}
            className="absolute inset-0 w-full h-80 p-4 bg-transparent text-transparent caret-white resize-none outline-none font-mono text-sm leading-6 z-20"
            style={{ color: 'transparent' }}
            spellCheck="false"
            data-testid="code-editor-textarea"
          />
          
          <pre
            ref={preRef}
            className="w-full h-80 p-4 bg-gray-900 text-gray-100 font-mono text-sm leading-6 overflow-auto whitespace-pre-wrap break-words pointer-events-none"
            aria-hidden="true"
          >
            <code className="syntax-highlight">
              {value || placeholder}
            </code>
          </pre>
        </div>
      </div>
    </div>
  );
}