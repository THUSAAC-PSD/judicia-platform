import React, { useState } from 'react';
import { apiRequest } from '../lib/queryClient';

interface BulkUserFormProps {
  onClose: () => void;
  onSuccess: () => void;
}

export default function BulkUserForm({ onClose, onSuccess }: BulkUserFormProps) {
  const [formData, setFormData] = useState({
    count: 10,
    prefix: 'contestant'
  });
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<any>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);

    try {
      const response = await apiRequest('/api/users/bulk-generate', {
        method: 'POST',
        body: JSON.stringify(formData)
      });
      
      setResult(response);
      onSuccess();
    } catch (error: any) {
      console.error('Failed to create bulk users:', error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg p-6 w-full max-w-md">
        <h2 className="text-xl font-bold mb-4">Bulk Create Users</h2>
        
        {!result ? (
          <form onSubmit={handleSubmit} className="space-y-4">
            <div>
              <label htmlFor="count" className="block text-sm font-medium text-gray-700 mb-1">
                Number of Users (1-100)
              </label>
              <input
                type="number"
                id="count"
                min="1"
                max="100"
                value={formData.count}
                onChange={(e) => setFormData(prev => ({ ...prev, count: parseInt(e.target.value) }))}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                required
              />
            </div>
            
            <div>
              <label htmlFor="prefix" className="block text-sm font-medium text-gray-700 mb-1">
                Username Prefix
              </label>
              <input
                type="text"
                id="prefix"
                value={formData.prefix}
                onChange={(e) => setFormData(prev => ({ ...prev, prefix: e.target.value }))}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                placeholder="contestant"
                required
              />
              <p className="text-xs text-gray-500 mt-1">
                Users will be created as: {formData.prefix}001, {formData.prefix}002, etc.
              </p>
            </div>
            
            <div className="bg-blue-50 p-3 rounded-md">
              <p className="text-sm text-blue-700">
                <strong>Default password:</strong> password123<br/>
                <strong>Email format:</strong> {formData.prefix}001@judicia.local<br/>
                <strong>Role:</strong> contestant
              </p>
            </div>
            
            <div className="flex justify-end space-x-3 pt-4">
              <button
                type="button"
                onClick={onClose}
                className="px-4 py-2 text-gray-700 border border-gray-300 rounded-md hover:bg-gray-50"
                disabled={loading}
              >
                Cancel
              </button>
              <button
                type="submit"
                className="px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 disabled:opacity-50"
                disabled={loading}
              >
                {loading ? 'Creating...' : `Create ${formData.count} Users`}
              </button>
            </div>
          </form>
        ) : (
          <div className="space-y-4">
            <div className="bg-green-50 p-4 rounded-md">
              <p className="text-green-700 font-medium">{result.message}</p>
            </div>
            
            <div>
              <h3 className="font-medium mb-2">Created Users:</h3>
              <div className="max-h-48 overflow-y-auto space-y-2">
                {result.users?.map((user: any) => (
                  <div key={user.id} className="bg-gray-50 p-2 rounded text-sm">
                    <strong>{user.username}</strong> - {user.email}
                  </div>
                ))}
              </div>
            </div>
            
            <div className="bg-yellow-50 p-3 rounded-md">
              <p className="text-yellow-700 text-sm">
                <strong>Important:</strong> All users have the default password: <code>{result.defaultPassword}</code>
              </p>
            </div>
            
            <div className="flex justify-end">
              <button
                onClick={onClose}
                className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700"
              >
                Close
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}