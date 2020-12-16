<template>
  <div id="app">
    <div>
      <input type="text" v-model="query">
    </div>
    <div v-for="(result, idx) in results" :key="idx">
        <i>{{result}}</i>
    </div>
  </div>
</template>

<script>
const axios = require('axios');
export default {
  name: 'app',
  data () {
    return {
      query: '',
      results: []
    }
  },
   watch: {
    query: function () {
      axios.get('http://127.0.0.1:8000/search', {
            params: {
              query: this.query
            }
          })
          .then(response => this.results = response.data["result"]);
    }
  }
}
</script>
