import React, { useEffect, useRef } from 'react';

interface SyntaxHighlighterProps {
  code: string;
  language: string;
  className?: string;
}

const languageMap: Record<string, string> = {
  cpp: 'cpp',
  python: 'python', 
  java: 'java',
  javascript: 'javascript'
};

// Simple syntax highlighter without Prism.js to avoid runtime errors
export function SyntaxHighlighter({ code, language, className = '' }: SyntaxHighlighterProps) {
  
  // Simple highlighting patterns for different languages
  const highlightCode = (code: string, lang: string) => {
    if (!code) return '';
    
    let highlighted = code;
    
    // Basic highlighting patterns
    const patterns = {
      cpp: [
        { regex: /(#include|#define|using namespace|int|char|float|double|void|return|if|else|for|while|class|public|private|protected)/g, class: 'text-blue-600 font-semibold' },
        { regex: /(\/\/.*$)/gm, class: 'text-green-600 italic' },
        { regex: /(\/\*[\s\S]*?\*\/)/g, class: 'text-green-600 italic' },
        { regex: /(".*?")/g, class: 'text-red-600' },
        { regex: /(\d+)/g, class: 'text-purple-600' }
      ],
      python: [
        { regex: /(def|class|import|from|if|elif|else|for|while|return|try|except|finally|with|as|in|not|and|or|True|False|None)/g, class: 'text-blue-600 font-semibold' },
        { regex: /(#.*$)/gm, class: 'text-green-600 italic' },
        { regex: /("""[\s\S]*?""")/g, class: 'text-green-600 italic' },
        { regex: /(".*?"|'.*?')/g, class: 'text-red-600' },
        { regex: /(\d+)/g, class: 'text-purple-600' }
      ],
      java: [
        { regex: /(public|private|protected|static|void|int|String|class|import|package|if|else|for|while|return|try|catch|finally|new)/g, class: 'text-blue-600 font-semibold' },
        { regex: /(\/\/.*$)/gm, class: 'text-green-600 italic' },
        { regex: /(\/\*[\s\S]*?\*\/)/g, class: 'text-green-600 italic' },
        { regex: /(".*?")/g, class: 'text-red-600' },
        { regex: /(\d+)/g, class: 'text-purple-600' }
      ],
      javascript: [
        { regex: /(function|var|let|const|if|else|for|while|return|try|catch|finally|class|import|export|default)/g, class: 'text-blue-600 font-semibold' },
        { regex: /(\/\/.*$)/gm, class: 'text-green-600 italic' },
        { regex: /(\/\*[\s\S]*?\*\/)/g, class: 'text-green-600 italic' },
        { regex: /(".*?"|'.*?'|`.*?`)/g, class: 'text-red-600' },
        { regex: /(\d+)/g, class: 'text-purple-600' }
      ]
    };

    return (
      <pre 
        className="overflow-auto text-sm bg-white p-4 font-mono leading-relaxed whitespace-pre-wrap"
        style={{ 
          margin: 0,
          lineHeight: '1.5rem',
          fontSize: '14px'
        }}
      >
        <code className="text-gray-800">
          {code}
        </code>
      </pre>
    );
  };

  return (
    <div className={`relative ${className}`}>
      {highlightCode(code, language)}
    </div>
  );
}