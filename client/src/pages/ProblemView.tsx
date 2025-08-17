import React, { useState } from 'react';
import { useParams, Link } from 'wouter';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { ChevronLeft, Play, FileText, TestTube, Upload, Download } from 'lucide-react';
import { apiRequest } from '../lib/queryClient';
import CodeEditor from '../components/CodeEditor';
import type { Problem, Submission } from '@shared/schema';

function Badge({ children, className }: { children: React.ReactNode; className: string }) {
  return (
    <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${className}`}>
      {children}
    </span>
  );
}

function TabButton({ 
  active, 
  onClick, 
  children, 
  icon: Icon 
}: { 
  active: boolean; 
  onClick: () => void; 
  children: React.ReactNode; 
  icon: React.ComponentType<{ className?: string }>;
}) {
  return (
    <button
      onClick={onClick}
      className={`flex items-center px-4 py-2 text-sm font-medium border-b-2 ${
        active 
          ? 'text-blue-600 border-blue-600' 
          : 'text-gray-500 border-transparent hover:text-gray-700 hover:border-gray-300'
      }`}
    >
      <Icon className="h-4 w-4 mr-2" />
      {children}
    </button>
  );
}

export default function ProblemView() {
  const { id } = useParams<{ id: string }>();
  const queryClient = useQueryClient();
  const [code, setCode] = useState('');
  const [language, setLanguage] = useState('cpp');
  const [activeTab, setActiveTab] = useState('description');
  const [uploadedFileName, setUploadedFileName] = useState<string | null>(null);

  const { data: problem, isLoading: problemLoading } = useQuery<Problem>({
    queryKey: ['/api/problems', id],
  });

  const { data: submissions } = useQuery<Submission[]>({
    queryKey: ['/api/submissions', 'problem', id],
  });

  const submitMutation = useMutation({
    mutationFn: async () => {
      if (!problem) throw new Error('Problem not loaded');
      
      let fileUrl = null;
      
      // If there's uploaded code, we'll save it as a file
      if (code && uploadedFileName) {
        // Get upload URL for the file
        const uploadResponse = await apiRequest('/api/objects/upload', {
          method: 'POST'
        });
        
        if (uploadResponse.uploadURL) {
          // Upload the code file
          const blob = new Blob([code], { type: 'text/plain' });
          await fetch(uploadResponse.uploadURL, {
            method: 'PUT',
            body: blob,
            headers: {
              'Content-Type': 'text/plain'
            }
          });
          
          fileUrl = uploadResponse.uploadURL;
        }
      }
      
      return apiRequest('/api/submissions', {
        method: 'POST',
        body: JSON.stringify({
          problemId: problem.id,
          contestId: problem.contestId,
          language,
          code,
          fileUrl
        }),
      });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['/api/submissions', 'problem', id] });
      setCode('');
      setUploadedFileName(null);
    },
  });

  const handleFileUpload = (content: string, filename: string) => {
    setCode(content);
    setUploadedFileName(filename);
    
    // Auto-detect language from file extension
    const extension = filename.split('.').pop()?.toLowerCase();
    const languageMap: Record<string, string> = {
      'cpp': 'cpp',
      'cc': 'cpp',
      'cxx': 'cpp',
      'c': 'c',
      'py': 'python',
      'java': 'java',
      'js': 'javascript',
      'ts': 'typescript',
      'go': 'go',
      'rs': 'rust',
      'cs': 'csharp'
    };
    
    if (extension && languageMap[extension]) {
      setLanguage(languageMap[extension]);
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'accepted': return 'bg-green-100 text-green-800';
      case 'wrong_answer': return 'bg-red-100 text-red-800';
      case 'time_limit_exceeded': return 'bg-yellow-100 text-yellow-800';
      case 'runtime_error': return 'bg-orange-100 text-orange-800';
      case 'compilation_error': return 'bg-purple-100 text-purple-800';
      case 'pending': return 'bg-blue-100 text-blue-800';
      default: return 'bg-gray-100 text-gray-800';
    }
  };

  const getDifficultyColor = (difficulty: string) => {
    switch (difficulty) {
      case 'Easy': return 'bg-green-100 text-green-800';
      case 'Medium': return 'bg-yellow-100 text-yellow-800';
      case 'Hard': return 'bg-red-100 text-red-800';
      default: return 'bg-gray-100 text-gray-800';
    }
  };

  if (problemLoading) {
    return (
      <div className="min-h-screen bg-gray-50 flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-32 w-32 border-b-2 border-blue-600"></div>
          <p className="mt-4 text-gray-600">Loading problem...</p>
        </div>
      </div>
    );
  }

  if (!problem) {
    return (
      <div className="min-h-screen bg-gray-50 flex items-center justify-center">
        <div className="text-center">
          <h1 className="text-2xl font-bold text-gray-900 mb-4">Problem Not Found</h1>
          <p className="text-gray-600 mb-8">The problem you're looking for doesn't exist.</p>
          <Link href="/contests" className="text-blue-600 hover:text-blue-800">
            ← Back to Contests
          </Link>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50">
      <div className="max-w-7xl mx-auto px-4 py-6">
        {/* Header */}
        <div className="mb-6">
          <Link href={`/contests/${problem.contestId}`} className="inline-flex items-center text-blue-600 hover:text-blue-800 mb-4">
            <ChevronLeft className="h-4 w-4 mr-1" />
            Back to Contest
          </Link>
          
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-3xl font-bold text-gray-900 mb-2">{problem.title}</h1>
              <div className="flex items-center space-x-4">
                <Badge className={getDifficultyColor(problem.difficulty)}>
                  {problem.difficulty}
                </Badge>
                <span className="text-sm text-gray-500">
                  Time Limit: {problem.timeLimit}ms
                </span>
                <span className="text-sm text-gray-500">
                  Memory Limit: {problem.memoryLimit}MB
                </span>
              </div>
            </div>
          </div>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          {/* Problem Description */}
          <div className="bg-white rounded-lg shadow-sm border border-gray-200">
            <div className="border-b border-gray-200">
              <nav className="-mb-px flex">
                <TabButton
                  active={activeTab === 'description'}
                  onClick={() => setActiveTab('description')}
                  icon={FileText}
                >
                  Description
                </TabButton>
                <TabButton
                  active={activeTab === 'submissions'}
                  onClick={() => setActiveTab('submissions')}
                  icon={TestTube}
                >
                  My Submissions
                </TabButton>
              </nav>
            </div>

            <div className="p-6">
              {activeTab === 'description' && (
                <div className="prose max-w-none">
                  <div className="whitespace-pre-wrap text-gray-700">
                    {problem.statement}
                  </div>
                  
                  {problem.sampleInput && (
                    <div className="mt-6">
                      <h3 className="text-lg font-semibold text-gray-900 mb-2">Sample Input</h3>
                      <pre className="bg-gray-50 p-4 rounded-lg border text-sm">
                        {problem.sampleInput}
                      </pre>
                    </div>
                  )}
                  
                  {problem.sampleOutput && (
                    <div className="mt-6">
                      <h3 className="text-lg font-semibold text-gray-900 mb-2">Sample Output</h3>
                      <pre className="bg-gray-50 p-4 rounded-lg border text-sm">
                        {problem.sampleOutput}
                      </pre>
                    </div>
                  )}
                </div>
              )}

              {activeTab === 'submissions' && (
                <div>
                  <h3 className="text-lg font-semibold text-gray-900 mb-4">Your Submissions</h3>
                  {submissions && submissions.length > 0 ? (
                    <div className="space-y-4">
                      {submissions.map((submission) => (
                        <div key={submission.id} className="border border-gray-200 rounded-lg p-4">
                          <div className="flex items-center justify-between mb-2">
                            <Badge className={getStatusColor(submission.status)}>
                              {submission.status.replace('_', ' ').toUpperCase()}
                            </Badge>
                            <span className="text-sm text-gray-500">
                              {submission.submittedAt ? new Date(submission.submittedAt).toLocaleString() : ''}
                            </span>
                          </div>
                          <div className="flex items-center space-x-4 text-sm text-gray-600">
                            <span>Language: {submission.language.toUpperCase()}</span>
                            <span>Score: {submission.score}/100</span>
                            <span>Time: {submission.executionTime}ms</span>
                            <span>Memory: {submission.memoryUsed}MB</span>
                          </div>
                        </div>
                      ))}
                    </div>
                  ) : (
                    <p className="text-gray-500">No submissions yet. Submit your solution to see results here.</p>
                  )}
                </div>
              )}
            </div>
          </div>

          {/* Code Editor */}
          <div className="bg-white rounded-lg shadow-sm border border-gray-200">
            <div className="border-b border-gray-200 p-4">
              <div className="flex items-center justify-between">
                <h2 className="text-lg font-semibold text-gray-900">Solution</h2>
                <div className="flex items-center space-x-3">
                  <select
                    value={language}
                    onChange={(e) => setLanguage(e.target.value)}
                    className="px-3 py-1.5 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                    data-testid="language-select"
                  >
                    <option value="cpp">C++</option>
                    <option value="c">C</option>
                    <option value="python">Python</option>
                    <option value="java">Java</option>
                    <option value="javascript">JavaScript</option>
                    <option value="go">Go</option>
                    <option value="rust">Rust</option>
                    <option value="csharp">C#</option>
                  </select>
                </div>
              </div>
              
              {uploadedFileName && (
                <div className="mt-2 flex items-center text-sm text-green-600">
                  <Upload className="w-4 h-4 mr-1" />
                  Uploaded: {uploadedFileName}
                </div>
              )}
            </div>

            <div className="p-4">
              <CodeEditor
                value={code}
                onChange={setCode}
                language={language}
                placeholder={`Write your ${language} solution here...`}
                onFileUpload={handleFileUpload}
                className="mb-4"
              />

              <div className="flex items-center justify-between">
                <div className="text-sm text-gray-500">
                  {code.split('\n').length} lines • {code.length} characters
                </div>
                <button
                  onClick={() => submitMutation.mutate()}
                  disabled={!code.trim() || submitMutation.isPending}
                  className="inline-flex items-center px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors"
                  data-testid="button-submit-solution"
                >
                  <Play className="h-4 w-4 mr-2" />
                  {submitMutation.isPending ? 'Submitting...' : 'Submit Solution'}
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}