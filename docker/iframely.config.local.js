(function() {
    var config = {

        // Specify a path for custom plugins. Custom plugins will override core plugins.
        // CUSTOM_PLUGINS_PATH: __dirname + '/yourcustom-plugin-folder',

        DEBUG: false,
        RICH_LOG_ENABLED: false,

        // For embeds that require render, baseAppUrl will be used as the host.
        baseAppUrl: "http://yourdomain.com",
        relativeStaticUrl: "/r",

        // Or just skip built-in renders altogether
        SKIP_IFRAMELY_RENDERS: true,

        // For legacy reasons the response format of Iframely open-source is
        // different by default as it does not group the links array by rel.
        // In order to get the same grouped response as in Cloud API,
        // add `&group=true` to your request to change response per request
        // or set `GROUP_LINKS` in your config to `true` for a global change.
        GROUP_LINKS: true,

        // Number of maximum redirects to follow before aborting the page
        // request with `redirect loop` error.
        MAX_REDIRECTS: 4,

        SKIP_OEMBED_RE_LIST: [
            // /^https?:\/\/yourdomain\.com\//,
        ],

        /*
        // Used to pass parameters to the generate functions when creating HTML elements
        // disableSizeWrapper: Don't wrap element (iframe, video, etc) in a positioned div
        GENERATE_LINK_PARAMS: {
            disableSizeWrapper: true
        },
        */

        port: 80, //can be overridden by PORT env var
        host: '0.0.0.0',    // Dockers beware. See https://github.com/itteco/iframely/issues/132#issuecomment-242991246
                            //can be overridden by HOST env var

        // Optional SSL cert, if you serve under HTTPS.
        /*
        ssl: {
            key: require('fs').readFileSync(__dirname + '/key.pem'),
            cert: require('fs').readFileSync(__dirname + '/cert.pem'),
            port: 443
        },
        */

        /*
        Supported cache engines:
        - no-cache - no caching will be used.
        - node-cache - good for debug, node memory will be used (https://github.com/tcs-de/nodecache).
        - redis - https://github.com/mranney/node_redis.
        - memcached - https://github.com/3rd-Eden/node-memcached
        */
        CACHE_ENGINE: 'node-cache',
        CACHE_TTL: 0, // In seconds.
        // 0 = 'never expire' for memcached & node-cache to let cache engine decide itself when to evict the record
        // 0 = 'no cache' for redis. Use high enough (e.g. 365*24*60*60*1000) ttl for similar 'never expire' approach instead

        /*
        // Redis cache options.
        REDIS_OPTIONS: {
            host: '127.0.0.1',
            port: 6379
        },
        */

        /*
        // Memcached options. See https://github.com/3rd-Eden/node-memcached#server-locations
        MEMCACHED_OPTIONS: {
            locations: "127.0.0.1:11211"
        }
        */

        /*
        // Access-Control-Allow-Origin list.
        allowedOrigins: [
            "*",
            "http://another_domain.com"
        ],
        */

        /*
        // Uncomment to enable plugin testing framework.
        tests: {
            mongodb: 'mongodb://localhost:27017/iframely-tests',
            single_test_timeout: 10 * 1000,
            plugin_test_period: 2 * 60 * 60 * 1000,
            relaunch_script_period: 5 * 60 * 1000
        },
        */

        // If there's no response from remote server, the timeout will occur after
        RESPONSE_TIMEOUT: 5 * 1000, //ms

        /* From v1.4.0, Iframely supports HTTP/2 by default. Disable it, if you'd rather not.
           Alternatively, you can also disable per origin. See `proxy` option below.
        */
        // DISABLE_HTTP2: true,

        // Customize API calls to oembed endpoints.
        ADD_OEMBED_PARAMS: [{
            // Endpoint url regexp array.
            re: [/^http:\/\/api\.instagram\.com\/oembed/],
            // Custom get params object.
            params: {
                hidecaption: true
            }
        }, {
            re: [/^https:\/\/www\.facebook\.com\/plugins\/page\/oembed\.json/i],
            params: {
                show_posts: 0,
                show_facepile: 0,
                maxwidth: 600
            }
        }, {
            // match i=user or i=moment or i=timeline to configure these types invidually
            // see params spec at https://dev.twitter.com/web/embedded-timelines/oembed
            re: [/^https?:\/\/publish\.twitter\.com\/oembed\?i=user/i],
            params: {
                limit: 1,
                maxwidth: 600
            }
        /*
        }, {
            // Facebook https://developers.facebook.com/docs/plugins/oembed-endpoints
            re: [/^https:\/\/www\.facebook\.com\/plugins\/\w+\/oembed\.json/i],
            params: {
                // Skip script tag and fb-root div.
                omitscript: true
            }
        */
         }],

        /*
        // Configure use of HTTP proxies as needed.
        // You don't have to specify all options per regex - just what you need to override
        PROXY: [{
            re: [/^https?:\/\/www\.domain\.com/],
            proxy_server: 'http://1.2.3.4:8080',
            user_agent: 'CHANGE YOUR AGENT',
            headers: {
                // HTTP headers
                // Overrides previous params if overlapped.
            },
            request_options: {
                // Refer to: https://github.com/request/request
                // Overrides previous params if overlapped.
            },
            disable_http2: true
        }],
        */

        // Customize API calls to 3rd parties. At the very least - configure required keys.
        providerOptions: {
            locale: "en_US",    // ISO 639-1 two-letter language code, e.g. en_CA or fr_CH.
                                // Will be added as highest priotity in accept-language header with each request.
                                // Plus is used in FB, YouTube and perhaps other plugins
            "twitter": {
                "max-width": 550,
                "min-width": 250,
                hide_media: false,
                hide_thread: false,
                omit_script: false,
                center: false,
                // dnt: true,
                cache_ttl: 100 * 365 * 24 * 3600 // 100 Years.
            },
            readability: {
                enabled: false
                // allowPTagDescription: true  // to enable description fallback to first paragraph
            },
            images: {
                loadSize: false, // if true, will try an load first bytes of all images to get/confirm the sizes
                checkFavicon: false // if true, will verify all favicons
            },
            tumblr: {
                consumer_key: "INSERT YOUR VALUE"
                // media_only: true     // disables status embeds for images and videos - will return plain media
            },
            google: {
                // https://developers.google.com/maps/documentation/embed/guide#api_key
                maps_key: "INSERT YOUR VALUE"
            },

            /*
            // Optional Camo Proxy to wrap all images: https://github.com/atmos/camo
            camoProxy: {
                camo_proxy_key: "INSERT YOUR VALUE",
                camo_proxy_host: "INSERT YOUR VALUE"
                // ssl_only: true // will only proxy non-ssl images
            },
            */

            // List of query parameters to add to YouTube and Vimeo frames
            // Start it with leading "?". Or omit alltogether for default values
            // API key is optional, youtube will work without it too.
            // It is probably the same API key you use for Google Maps.
            youtube: {
                // api_key: "INSERT YOUR VALUE",
                get_params: "?rel=0&showinfo=1"     // https://developers.google.com/youtube/player_parameters
            },
            vimeo: {
                get_params: "?byline=0&badge=0"     // https://developer.vimeo.com/player/embedding
            },

            /*
            soundcloud: {
                old_player: true // enables classic player
            },
            giphy: {
                media_only: true // disables branded player for gifs and returns just the image
            }
            */
            /*
            bandcamp: {
                get_params: '/size=large/bgcol=333333/linkcol=ffffff/artwork=small/transparent=true/',
                media: {
                    album: {
                        height: 472,
                        'max-width': 700
                    },
                    track: {
                        height: 120,
                        'max-width': 700
                    }
                }
            }
            */
        },

        // WHITELIST_WILDCARD, if present, will be added to whitelist as record for top level domain: "*"
        // with it, you can define what parsers do when they run accross unknown publisher.
        // If absent or empty, all generic media parsers will be disabled except for known domains
        // More about format: https://iframely.com/docs/qa-format

        /*
        WHITELIST_WILDCARD: {
              "twitter": {
                "player": "allow",
                "photo": "deny"
              },
              "oembed": {
                "video": "allow",
                "photo": "allow",
                "rich": "deny",
                "link": "deny"
              },
              "og": {
                "video": ["allow", "ssl", "responsive"]
              },
              "iframely": {
                "survey": "allow",
                "reader": "allow",
                "player": "allow",
                "image": "allow"
              },
              "html-meta": {
                "video": ["allow", "responsive"],
                "promo": "allow"
              }
        }
        */

        // Black-list any of the inappropriate domains. Iframely will return 417
        // At minimum, keep your localhosts blacklisted to avoid SSRF
        BLACKLIST_DOMAINS_RE: [
            /^https?:\/\/127\.0\.0\.1/i,
            /^https?:\/\/localhost/i,

            // And this is AWS metadata service
            // https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/ec2-instance-metadata.html
            /^https?:\/\/169\.254\.169\.254/
        ]
    };

    module.exports = config;
})();
