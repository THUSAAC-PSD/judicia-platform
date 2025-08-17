import React from 'react';
import { useParams, Link } from 'wouter';
import { useQuery } from '@tanstack/react-query';
import { Clock, Users, Trophy, Calendar } from 'lucide-react';
import { Contest, Problem } from '@shared/schema';

function getDifficultyColor(difficulty: string) {
  switch (difficulty) {
    case 'easy':
      return 'bg-green-100 text-green-800';
    case 'medium':
      return 'bg-yellow-100 text-yellow-800';
    case 'hard':
      return 'bg-red-100 text-red-800';
    default:
      return 'bg-gray-100 text-gray-800';
  }
}

function getStatusBadge(status: string) {
  switch (status) {
    case 'solved':
      return <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800">Solved</span>;
    case 'attempted':
      return <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800">Attempted</span>;
    case 'not_attempted':
    default:
      return <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-800">Not Attempted</span>;
  }
}

function Badge({ children, className }: { children: React.ReactNode; className: string }) {
  return (
    <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${className}`}>
      {children}
    </span>
  );
}

function ProblemRow({ problem, index }: { problem: Problem; index: number }) {
  const letter = String.fromCharCode(65 + index); // A, B, C, etc.
  
  return (
    <tr className="hover:bg-gray-50 cursor-pointer" data-testid={`row-problem-${problem.id}`}>
      <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
        {letter}
      </td>
      <td className="px-6 py-4 whitespace-nowrap">
        <Link href={`/problems/${problem.id}`} className="block">
          <div className="text-sm font-medium text-gray-900 hover:text-blue-600" data-testid={`text-problem-title-${problem.id}`}>
            {problem.title}
          </div>
          <div className="text-sm text-gray-500 truncate max-w-md">
            {problem.statement.split('\n')[0].substring(0, 100)}...
          </div>
        </Link>
      </td>
      <td className="px-6 py-4 whitespace-nowrap">
        <Badge className={getDifficultyColor(problem.difficulty)}>
          {problem.difficulty.charAt(0).toUpperCase() + problem.difficulty.slice(1)}
        </Badge>
      </td>
      <td className="px-6 py-4 whitespace-nowrap">
        {getStatusBadge('not_attempted')} {/* TODO: Get actual status */}
      </td>
      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
        {problem.points}
      </td>
      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
        1,234 {/* TODO: Get actual solve count */}
      </td>
    </tr>
  );
}

export default function ContestDetail() {
  const { id } = useParams<{ id: string }>();
  
  const { data: contest, isLoading: contestLoading } = useQuery<Contest>({
    queryKey: ['/api/contests', id],
    enabled: !!id
  });

  const { data: problems = [], isLoading: problemsLoading } = useQuery<Problem[]>({
    queryKey: ['/api/contests', id, 'problems'],
    enabled: !!id
  });

  if (contestLoading) {
    return (
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="animate-pulse">
          <div className="h-8 bg-gray-200 rounded w-1/3 mb-4"></div>
          <div className="h-4 bg-gray-200 rounded w-2/3 mb-8"></div>
          <div className="grid grid-cols-1 md:grid-cols-4 gap-6 mb-8">
            {[...Array(4)].map((_, i) => (
              <div key={i} className="bg-white p-6 rounded-lg border">
                <div className="h-4 bg-gray-200 rounded w-1/2 mb-2"></div>
                <div className="h-6 bg-gray-200 rounded w-3/4"></div>
              </div>
            ))}
          </div>
        </div>
      </div>
    );
  }

  if (!contest) {
    return (
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="text-center">
          <h1 className="text-2xl font-bold text-gray-900">Contest not found</h1>
          <p className="mt-2 text-gray-600">The contest you're looking for doesn't exist.</p>
          <Link href="/contests" className="mt-4 inline-block text-blue-600 hover:text-blue-500">
            ← Back to contests
          </Link>
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      {/* Contest Header */}
      <div className="mb-8">
        <div className="flex items-center justify-between mb-4">
          <div>
            <h1 className="text-3xl font-bold text-gray-900" data-testid="text-contest-title">
              {contest.title}
            </h1>
            <p className="mt-2 text-gray-600" data-testid="text-contest-description">
              {contest.description}
            </p>
          </div>
          <Link 
            href="/contests" 
            className="px-4 py-2 text-sm text-gray-600 hover:text-gray-800 border border-gray-300 rounded-md hover:bg-gray-50"
            data-testid="link-back-contests"
          >
            ← Back to contests
          </Link>
        </div>

        {/* Contest Stats */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-6 mb-8">
          <div className="bg-white p-6 rounded-lg border border-gray-200 shadow-sm">
            <div className="flex items-center">
              <Calendar className="h-5 w-5 text-gray-400" />
              <div className="ml-3">
                <p className="text-sm font-medium text-gray-500">Start Time</p>
                <p className="text-lg font-semibold text-gray-900">
                  {new Date(contest.startTime).toLocaleDateString()}
                </p>
              </div>
            </div>
          </div>

          <div className="bg-white p-6 rounded-lg border border-gray-200 shadow-sm">
            <div className="flex items-center">
              <Clock className="h-5 w-5 text-gray-400" />
              <div className="ml-3">
                <p className="text-sm font-medium text-gray-500">Duration</p>
                <p className="text-lg font-semibold text-gray-900">
                  {(() => {
                    const start = new Date(contest.startTime).getTime();
                    const end = new Date(contest.endTime).getTime();
                    const seconds = Math.max(0, Math.floor((end - start) / 1000));
                    const h = Math.floor(seconds / 3600);
                    const m = Math.floor((seconds % 3600) / 60);
                    return `${h}h ${m}m`;
                  })()}
                </p>
              </div>
            </div>
          </div>

          <div className="bg-white p-6 rounded-lg border border-gray-200 shadow-sm">
            <div className="flex items-center">
              <Users className="h-5 w-5 text-gray-400" />
              <div className="ml-3">
                <p className="text-sm font-medium text-gray-500">Participants</p>
                <p className="text-lg font-semibold text-gray-900">N/A</p>
              </div>
            </div>
          </div>

          <div className="bg-white p-6 rounded-lg border border-gray-200 shadow-sm">
            <div className="flex items-center">
              <Trophy className="h-5 w-5 text-gray-400" />
              <div className="ml-3">
                <p className="text-sm font-medium text-gray-500">Status</p>
                <p className="text-lg font-semibold text-gray-900">
                  {new Date() < new Date(contest.startTime) ? 'Upcoming' :
                   new Date() > new Date(contest.endTime) ? 'Ended' : 'Running'}
                </p>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Problems Section */}
      <div className="bg-white shadow-sm rounded-lg border border-gray-200">
        <div className="px-6 py-4 border-b border-gray-200">
          <h2 className="text-lg font-semibold text-gray-900">
            Problems ({problems.length})
          </h2>
        </div>
        
        <div className="overflow-hidden">
          {problemsLoading ? (
            <div className="p-6">
              <div className="animate-pulse space-y-4">
                {[...Array(3)].map((_, i) => (
                  <div key={i} className="flex space-x-4">
                    <div className="w-8 h-4 bg-gray-200 rounded"></div>
                    <div className="flex-1 space-y-2">
                      <div className="h-4 bg-gray-200 rounded w-1/4"></div>
                      <div className="h-3 bg-gray-200 rounded w-3/4"></div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          ) : problems.length === 0 ? (
            <div className="text-center py-12">
              <p className="text-gray-500">No problems available in this contest.</p>
            </div>
          ) : (
            <div className="overflow-x-auto">
              <table className="min-w-full divide-y divide-gray-200">
                <thead className="bg-gray-50">
                  <tr>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      #
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Problem
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Difficulty
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Status
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Points
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Solved
                    </th>
                  </tr>
                </thead>
                <tbody className="bg-white divide-y divide-gray-200">
                  {problems.map((problem, index) => (
                    <ProblemRow key={problem.id} problem={problem} index={index} />
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}