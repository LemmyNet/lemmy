import { Component, linkEvent } from 'inferno';
import { Subscription } from "rxjs";
import { retryWhen, delay, take } from 'rxjs/operators';
import { UserOperation, Post, Comment, SortType, SearchForm, SearchResponse, SearchType } from '../interfaces';
import { WebSocketService } from '../services';
import { msgOp, fetchLimit } from '../utils';
import { PostListing } from './post-listing';
import { CommentNodes } from './comment-nodes';

interface SearchState {
  q: string,
  type_: SearchType,
  sort: SortType,
  page: number,
  searchResponse: SearchResponse;
  loading: boolean;
}

export class Search extends Component<any, SearchState> {

  private subscription: Subscription;
  private emptyState: SearchState = {
    q: undefined,
    type_: SearchType.Both,
    sort: SortType.TopAll,
    page: 1,
    searchResponse: {
      op: null,
      posts: [],
      comments: [],
    },
    loading: false,
  }

  constructor(props: any, context: any) {
    super(props, context);

    this.state = this.emptyState;

    this.subscription = WebSocketService.Instance.subject
    .pipe(retryWhen(errors => errors.pipe(delay(3000), take(10))))
    .subscribe(
      (msg) => this.parseMessage(msg),
        (err) => console.error(err),
        () => console.log('complete')
    );

  }

  componentWillUnmount() {
    this.subscription.unsubscribe();
  }

  componentDidMount() {
    document.title = "Search - Lemmy";
  }

  render() {
    return (
      <div class="container">
        <div class="row">
          <div class="col-12">
            <h5>Search</h5>
            {this.selects()}
            {this.searchForm()}
            {this.state.type_ == SearchType.Both &&
              this.both()
            }
            {this.state.type_ == SearchType.Comments &&
              this.comments()
            }
            {this.state.type_ == SearchType.Posts &&
              this.posts()
            }
            {this.noResults()}
            {this.paginator()}
          </div>
        </div>
      </div>
    )
  }

  searchForm() {
    return (
      <form class="form-inline" onSubmit={linkEvent(this, this.handleSearchSubmit)}>
        <input type="text" class="form-control mr-2" value={this.state.q} placeholder="Search..." onInput={linkEvent(this, this.handleQChange)} required minLength={3} />
        <button type="submit" class="btn btn-secondary mr-2">
          {this.state.loading ?
          <svg class="icon icon-spinner spin"><use xlinkHref="#icon-spinner"></use></svg> :
          <span>Search</span>
          }
        </button>
      </form>
    )
  }

  selects() {
    return (
      <div className="mb-2">
        <select value={this.state.type_} onChange={linkEvent(this, this.handleTypeChange)} class="custom-select custom-select-sm w-auto">
          <option disabled>Type</option>
          <option value={SearchType.Both}>Both</option>
          <option value={SearchType.Comments}>Comments</option>
          <option value={SearchType.Posts}>Posts</option>
        </select>
        <select value={this.state.sort} onChange={linkEvent(this, this.handleSortChange)} class="custom-select custom-select-sm w-auto ml-2">
          <option disabled>Sort Type</option>
          <option value={SortType.New}>New</option>
          <option value={SortType.TopDay}>Top Day</option>
          <option value={SortType.TopWeek}>Week</option>
          <option value={SortType.TopMonth}>Month</option>
          <option value={SortType.TopYear}>Year</option>
          <option value={SortType.TopAll}>All</option>
        </select>
      </div>
    )

  }

  both() {
    let combined: Array<{type_: string, data: Comment | Post}> = [];
    let comments = this.state.searchResponse.comments.map(e => {return {type_: "comments", data: e}});
    let posts = this.state.searchResponse.posts.map(e => {return {type_: "posts", data: e}});

    combined.push(...comments);
    combined.push(...posts);

    // Sort it
    if (this.state.sort == SortType.New) {
      combined.sort((a, b) => b.data.published.localeCompare(a.data.published));
    } else {
      combined.sort((a, b) => b.data.score - a.data.score);
    }

    return (
      <div>
        {combined.map(i =>
          <div>
            {i.type_ == "posts"
              ? <PostListing post={i.data as Post} showCommunity viewOnly />
              : <CommentNodes nodes={[{comment: i.data as Comment}]} viewOnly noIndent />
            }
          </div>
                     )
        }
      </div>
    )
  }

  comments() {
    return (
      <div>
        {this.state.searchResponse.comments.map(comment => 
          <CommentNodes nodes={[{comment: comment}]} noIndent viewOnly />
        )}
      </div>
    );
  }

  posts() {
    return (
      <div>
        {this.state.searchResponse.posts.map(post => 
          <PostListing post={post} showCommunity viewOnly />
        )}
      </div>
    );
  }

  paginator() {
    return (
      <div class="mt-2">
        {this.state.page > 1 && 
          <button class="btn btn-sm btn-secondary mr-1" onClick={linkEvent(this, this.prevPage)}>Prev</button>
        }
        <button class="btn btn-sm btn-secondary" onClick={linkEvent(this, this.nextPage)}>Next</button>
      </div>
    );
  }

  noResults() {
    let res = this.state.searchResponse;
    return (
      <div>
        {res && res.op && res.posts.length == 0 && res.comments.length == 0 && 
          <span>No Results</span>
        }
      </div>
    )
  }

  nextPage(i: Search) { 
    i.state.page++;
    i.setState(i.state);
    i.search();
  }

  prevPage(i: Search) { 
    i.state.page--;
    i.setState(i.state);
    i.search();
  }

  search() {
    // TODO community
    let form: SearchForm = {
      q: this.state.q,
      type_: SearchType[this.state.type_],
      sort: SortType[this.state.sort],
      page: this.state.page,
      limit: fetchLimit,
    };

    WebSocketService.Instance.search(form);
  }

  handleSortChange(i: Search, event: any) {
    i.state.sort = Number(event.target.value);
    i.state.page = 1;
    i.setState(i.state);
    i.search();
  }

  handleTypeChange(i: Search, event: any) {
    i.state.type_ = Number(event.target.value);
    i.state.page = 1;
    i.setState(i.state);
    i.search();
  }

  handleSearchSubmit(i: Search, event: any) {
    event.preventDefault();
    i.state.loading = true;
    i.search();
    i.setState(i.state);
  }

  handleQChange(i: Search, event: any) {
    i.state.q = event.target.value;
    i.setState(i.state);
  }

  parseMessage(msg: any) {
    console.log(msg);
    let op: UserOperation = msgOp(msg);
    if (msg.error) {
      alert(msg.error);
      return;
    } else if (op == UserOperation.Search) {
      let res: SearchResponse = msg;
      this.state.searchResponse = res;
      this.state.loading = false;
      document.title = `Search - ${this.state.q} - Lemmy`;
      this.setState(this.state);
      
    }
  }
}

