// @flow

import axios from 'axios';
import type { $AxiosError } from 'axios';
import { appListen } from '../src/server';

axios.defaults.baseURL = 'http://localhost:3000';

beforeAll(() => appListen);

describe('App responds on requests', () => {
  it('GET / returns 200', done => {
    axios.get('/').then(response => {
      expect(response.status).toEqual(200);
      done();
    });
  });
  it('PUT / returns 404', done => {
    axios.put('/').catch((error: $AxiosError<mixed>) => {
      if (!error.response) throw new Error();

      expect(error.response.status).toEqual(404);
      done();
    });
  });
});