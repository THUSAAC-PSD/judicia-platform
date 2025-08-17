import React from 'react';
import { Link } from 'wouter';
import { useQuery } from '@tanstack/react-query';
import { Calendar, Clock, Users, Plus } from 'lucide-react';
import { useAuth } from '@/lib/auth';
import type { Contest } from '@shared/schema';

function ContestCard({ contest }: { contest: Contest }) {
  const startDate = new Date(contest.startTime);
  const endDate = new Date(contest.endTime);
  const now = new Date();
  
  const getStatus = () => {
    if (now < startDate) return { text: 'Upcoming', color: 'bg-blue-100 text-blue-800' };
    if (now > endDate) return { text: 'Ended', color: 'bg-gray-100 text-gray-800' };
    return { text: 'Running', color: 'bg-green-100 text-green-800' };
  };

  const status = getStatus();

  return (
    <Link href={`/contests/${contest.id}`}>
      <div 
        className="bg-white rounded-lg border border-gray-200 shadow-sm hover:shadow-md transition-shadow cursor-pointer p-6"
        data-testid={`card-contest-${contest.id}`}
      >
        <div className="flex items-start justify-between mb-4">
          <div className="flex-1">
            <h3 className="text-lg font-semibold text-gray-900 mb-2" data-testid={`text-contest-title-${contest.id}`}>
              {contest.title}
            </h3>
            <p className="text-sm text-gray-600 line-clamp-2">
              {contest.description}
            </p>
          </div>
          <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${status.color}`}>
            {status.text}
          </span>
        </div>
        
        <div className="space-y-2 text-sm text-gray-500">
          <div className="flex items-center">
            <Calendar className="h-4 w-4 mr-2" />
            <span>
              {startDate.toLocaleDateString()} - {endDate.toLocaleDateString()}
            </span>
          </div>
          
          <div className="flex items-center">
            <Clock className="h-4 w-4 mr-2" />
            <span>
              {startDate.toLocaleTimeString()} - {endDate.toLocaleTimeString()}
            </span>
          </div>
          
          <div className="flex items-center">
            <Users className="h-4 w-4 mr-2" />
            <span>Participants: N/A</span>
          </div>
        </div>
      </div>
    </Link>
  );
}

export default function ContestList() {
  const { user } = useAuth();
  const { data: contests = [], isLoading } = useQuery<Contest[]>({
    queryKey: ['/api/contests']
  });

  if (isLoading) {
    return (
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="animate-pulse">
          <div className="flex justify-between items-center mb-8">
            <div className="h-8 bg-gray-200 rounded w-48"></div>
            <div className="h-10 bg-gray-200 rounded w-32"></div>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {[...Array(6)].map((_, i) => (
              <div key={i} className="bg-white rounded-lg border p-6">
                <div className="space-y-4">
                  <div className="h-6 bg-gray-200 rounded w-3/4"></div>
                  <div className="h-4 bg-gray-200 rounded w-full"></div>
                  <div className="h-4 bg-gray-200 rounded w-2/3"></div>
                  <div className="space-y-2">
                    <div className="h-3 bg-gray-200 rounded w-1/2"></div>
                    <div className="h-3 bg-gray-200 rounded w-1/3"></div>
                    <div className="h-3 bg-gray-200 rounded w-1/4"></div>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      <div className="flex justify-between items-center mb-8">
        <div>
          <h1 className="text-3xl font-bold text-gray-900">Contests</h1>
          <p className="mt-2 text-gray-600">
            Participate in programming contests and improve your skills
          </p>
        </div>
        
  {user && user.roles.includes('admin') && (
          <Link
            href="/admin"
            className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
            data-testid="link-admin-panel"
          >
            <Plus className="h-4 w-4 mr-2" />
            Create Contest
          </Link>
        )}
      </div>

      {contests.length === 0 ? (
        <div className="text-center py-12">
          <h3 className="text-lg font-medium text-gray-900 mb-2">No contests available</h3>
          <p className="text-gray-600">
            {user && user.roles.includes('admin') 
              ? 'Create your first contest to get started' 
              : 'Check back later for new contests'}
          </p>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {contests.map((contest) => (
            <ContestCard key={contest.id} contest={contest} />
          ))}
        </div>
      )}
    </div>
  );
}